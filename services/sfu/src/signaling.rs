use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, error};
use std::sync::Arc;

use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};

use crate::SFU;

#[derive(Serialize, Deserialize, Debug, Message, Clone)]
#[rtype(result = "()")]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SignalingMessage {
    #[serde(rename_all = "kebab-case")]
    Join { room_id: Uuid, peer_id: Uuid, role: String },
    #[serde(rename_all = "kebab-case")]
    Offer { sdp: String, peer_id: Uuid },
    #[serde(rename_all = "kebab-case")]
    Answer { sdp: String, peer_id: Uuid },
    #[serde(rename_all = "kebab-case")]
    IceCandidate { candidate: serde_json::Value, peer_id: Uuid },
    #[serde(rename_all = "kebab-case")]
    NewTrack { track_id: Uuid, peer_id: Uuid },
    #[serde(rename_all = "kebab-case")]
    Error { message: String },
}

pub struct SignalingSession {
    pub id: Uuid,
    pub room_id: Option<Uuid>,
    pub sfu: web::Data<SFU>,
    pub pc: Option<Arc<RTCPeerConnection>>,
}

impl Actor for SignalingSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Signaling session started: {}", self.id);
        self.sfu.sessions.insert(self.id, ctx.address());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Signaling session stopped: {}", self.id);
        self.sfu.sessions.remove(&self.id);
        if let Some(pc) = self.pc.take() {
            tokio::spawn(async move { let _ = pc.close().await; });
        }
    }
}

impl Handler<SignalingMessage> for SignalingSession {
    type Result = ();
    fn handle(&mut self, msg: SignalingMessage, ctx: &mut Self::Context) {
        if let Ok(text) = serde_json::to_string(&msg) {
            ctx.text(text);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SignalingSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<SignalingMessage>(&text) {
                    Ok(signaling_msg) => self.handle_signaling_message(signaling_msg, ctx),
                    Err(e) => error!("Parse error: {:?}. Raw: {}", e, text),
                }
            }
            Ok(ws::Message::Close(reason)) => ctx.close(reason),
            _ => (),
        }
    }
}

impl SignalingSession {
    fn handle_signaling_message(&mut self, msg: SignalingMessage, ctx: &mut ws::WebsocketContext<Self>) {
        let sfu_data = self.sfu.clone();
        let addr = ctx.address();
        let peer_id = self.id;

        match msg {
            SignalingMessage::Join { room_id, peer_id: id, role: _ } => {
                self.id = id;
                self.room_id = Some(room_id);
                self.sfu.sessions.insert(id, addr.clone());
                
                let sfu_clone = sfu_data.get_ref().clone();
                let addr_clone = addr.clone();
                ctx.spawn(async move {
                    if sfu_clone.check_room_exists(room_id).await {
                        sfu_clone.register_session(room_id, addr_clone).await;
                    } else {
                        addr_clone.do_send(SignalingMessage::Error { message: "Room not found".to_string() });
                    }
                }.into_actor(self));
            }
            
            SignalingMessage::Offer { sdp, peer_id: _ } => {
                let sfu_clone = sfu_data.clone();
                let addr_clone = addr.clone();
                let room_id = self.room_id;

                ctx.spawn(async move {
                    let sfu = sfu_clone.get_ref();
                    if let Ok(pc) = sfu.create_webrtc_pc().await {
                        let pc = Arc::new(pc);
                        let pc_inner = pc.clone();
                        let addr_inner = addr_clone.clone();
                        let sfu_inner = sfu.clone();

                        pc.on_track(Box::new(move |track, _receiver, _| {
                            let track_id = Uuid::new_v4();
                            info!("New track received from peer {}: {:?}", peer_id, track.id());
                            
                            let sfu_relay = sfu_inner.clone();
                            let room_id_relay = room_id;
                            let addr_relay = addr_inner.clone(); 
                            
                            Box::pin(async move {
                                let local_track = Arc::new(TrackLocalStaticRTP::new(
                                    track.codec().capability,
                                    "video".to_string(),
                                    peer_id.to_string(),
                                ));
                                
                                sfu_relay.relay_tracks.insert(track_id, local_track.clone());

                                if let Some(rid) = room_id_relay {
                                    if let Some(sessions) = sfu_relay.room_sessions.get(&rid) {
                                        for session_addr in sessions.value() {
                                            if session_addr != &addr_relay {
                                                session_addr.do_send(SignalingMessage::NewTrack { track_id, peer_id });
                                            }
                                        }
                                    }
                                }

                                while let Ok((rtp, _)) = track.read_rtp().await {
                                    let _ = local_track.write_rtp(&rtp).await;
                                }
                            })
                        }));

                        if let Ok(desc) = RTCSessionDescription::offer(sdp) {
                            if pc.set_remote_description(desc).await.is_ok() {
                                if let Ok(answer) = pc.create_answer(None).await {
                                    let _ = pc.set_local_description(answer.clone()).await;
                                    addr_clone.do_send(SignalingMessage::Answer { sdp: answer.sdp, peer_id });
                                }
                            }
                        }
                        addr_clone.do_send(InternalMsg { pc: pc_inner });
                    }
                }.into_actor(self));
            }

            SignalingMessage::NewTrack { track_id, peer_id: publisher_id } => {
                if let Some(pc) = &self.pc {
                    if let Some(local_track) = self.sfu.relay_tracks.get(&track_id) {
                        let pc_clone = pc.clone();
                        let local_track_val = local_track.value().clone();
                        let addr_clone = addr.clone();
                        
                        ctx.spawn(async move {
                            // Correction : conversion explicite vers le trait dynamique
                            let track: Arc<dyn TrackLocal + Send + Sync> = local_track_val;
                            if let Ok(_) = pc_clone.add_track(track).await {
                                if let Ok(offer) = pc_clone.create_offer(None).await {
                                    let _ = pc_clone.set_local_description(offer.clone()).await;
                                    addr_clone.do_send(SignalingMessage::Offer { sdp: offer.sdp, peer_id: publisher_id });
                                }
                            }
                        }.into_actor(self));
                    }
                }
            }
            
            _ => {}
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct InternalMsg { pub pc: Arc<RTCPeerConnection> }

impl Handler<InternalMsg> for SignalingSession {
    type Result = ();
    fn handle(&mut self, msg: InternalMsg, _ctx: &mut Self::Context) { self.pc = Some(msg.pc); }
}

pub async fn ws_endpoint(
    req: HttpRequest,
    stream: web::Payload,
    sfu: web::Data<SFU>,
) -> Result<HttpResponse, Error> {
    ws::start(SignalingSession { id: Uuid::new_v4(), room_id: None, sfu, pc: None }, &req, stream)
}
