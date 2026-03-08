export interface Participant {
  id: string
  name: string
  videoTrack?: MediaStreamTrack
  audioTrack?: MediaStreamTrack
  isLocal: boolean
}

export class WebRTCManager {
  private ws: WebSocket | null = null;
  private pc: RTCPeerConnection | null = null;
  private localStream: MediaStream | null = null;
  private participants: Map<string, Participant> = new Map();
  private peerId: string = crypto.randomUUID();
  private isJoining: boolean = false;
  private onParticipantsUpdate: (participants: Participant[]) => void = () => {};

  constructor(onUpdate: (p: Participant[]) => void) {
    this.onParticipantsUpdate = onUpdate;
  }

  async joinRoom(roomId: string, userName: string, mode: 'publisher' | 'viewer' = 'publisher'): Promise<void> {
    if (this.isJoining) return;
    this.isJoining = true;
    
    try {
        if (mode === 'publisher') {
            this.localStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        }

        const localParticipant: Participant = {
            id: this.peerId,
            name: userName,
            videoTrack: this.localStream?.getVideoTracks()[0],
            isLocal: true,
        };
        this.participants.set(this.peerId, localParticipant);
        this.onParticipantsUpdate(this.getParticipants());

        this.ws = new WebSocket(`ws://${window.location.hostname}:5004/ws`);

        this.ws.onmessage = async (event) => {
            const msg = JSON.parse(event.data);
            console.log('📩 Signal:', msg.type);

            if (msg.type === 'offer') {
                // Offre venant du serveur (nouveau flux distant)
                await this.pc?.setRemoteDescription(new RTCSessionDescription(msg));
                const answer = await this.pc?.createAnswer();
                await this.pc?.setLocalDescription(answer);
                this.send({ type: 'answer', sdp: answer?.sdp, 'peer-id': this.peerId });
            } 
            else if (msg.type === 'answer') {
                await this.pc?.setRemoteDescription(new RTCSessionDescription(msg));
            } 
            else if (msg.type === 'ice-candidate') {
                await this.pc?.addIceCandidate(new RTCIceCandidate(msg.candidate));
            }
            else if (msg.type === 'new-track') {
                // Information sur un nouveau participant
                if (!this.participants.has(msg['peer-id'])) {
                    this.participants.set(msg['peer-id'], {
                        id: msg['peer-id'],
                        name: `Participant ${msg['peer-id'].slice(0, 4)}`,
                        isLocal: false
                    });
                    this.onParticipantsUpdate(this.getParticipants());
                }
            }
        };

        this.ws.onopen = () => {
            this.send({ type: 'join', 'room-id': roomId, 'peer-id': this.peerId, 'role': mode });
            this.startNegotiation(mode);
        };

    } catch (e) {
        this.isJoining = false;
        throw e;
    }
  }

  private async startNegotiation(mode: 'publisher' | 'viewer') {
    this.pc = new RTCPeerConnection({ iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] });

    this.pc.onicecandidate = (e) => {
        if (e.candidate) this.send({ type: 'ice-candidate', candidate: e.candidate, 'peer-id': this.peerId });
    };

    this.pc.ontrack = (e) => {
        console.log('🎥 Received Remote Track');
        // On cherche à quel participant appartient ce flux (simplifié pour le proto)
        // Dans un vrai SFU on mapperait via msid
        const remoteParticipant = Array.from(this.participants.values()).find(p => !p.isLocal && !p.videoTrack);
        if (remoteParticipant) {
            remoteParticipant.videoTrack = e.track;
            this.onParticipantsUpdate(this.getParticipants());
        }
    };

    if (mode === 'publisher' && this.localStream) {
        this.localStream.getTracks().forEach(t => this.pc?.addTrack(t, this.localStream!));
    }

    const offer = await this.pc.createOffer();
    await this.pc.setLocalDescription(offer);
    this.send({ type: 'offer', sdp: offer.sdp, 'peer-id': this.peerId });
  }

  private send(msg: any) { this.ws?.send(JSON.stringify(msg)); }

  getParticipants(): Participant[] { return Array.from(this.participants.values()); }
}
