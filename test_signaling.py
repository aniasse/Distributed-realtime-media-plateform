import asyncio
import websockets
import json
import uuid

async def test_full_negotiation():
    uri = "ws://localhost:5004/ws"
    peer_id = str(uuid.uuid4())
    room_id = str(uuid.uuid4())
    
    print(f"🚀 Connexion à {uri}...")
    try:
        async with websockets.connect(uri) as websocket:
            print("✅ Connecté au SFU !")
            
            # 1. Étape de JOIN
            join_msg = {
                "type": "join",
                "room-id": room_id,
                "peer-id": peer_id
            }
            print(f"📤 Envoi Join...")
            await websocket.send(json.dumps(join_msg))
            
            # 2. Étape d'OFFER (Simulation SDP)
            mock_offer_sdp = (
                "v=0\r\no=- 47283 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n"
                "m=video 9 UDP/TLS/RTP/SAVPF 96\r\n"
                "a=rtpmap:96 VP8/90000\r\n"
            )
            offer_msg = {
                "type": "offer",
                "sdp": mock_offer_sdp,
                "peer-id": peer_id
            }
            print(f"📤 Envoi Offer SDP...")
            await websocket.send(json.dumps(offer_msg))
            
            # 3. Attente de l'ANSWER du serveur
            print("⏳ Attente de la réponse du SFU...")
            response_json = await websocket.recv()
            response = json.loads(response_json)
            
            if response.get("type") == "answer":
                print("💎 RÉUSSITE : Réponse SDP reçue du serveur !")
                print(f"📄 SDP de réponse :\n{response.get('sdp')}")
            else:
                print(f"⚠️ Message inattendu : {response}")
                
            # 4. Envoi d'un candidat ICE (simulation)
            ice_msg = {
                "type": "ice-candidate",
                "candidate": {"candidate": "123456", "sdpMid": "0"},
                "peer-id": peer_id
            }
            print(f"📤 Envoi ICE Candidate...")
            await websocket.send(json.dumps(ice_msg))
            
            await asyncio.sleep(1)
            print("🏁 Test de négociation terminé.")
            
    except Exception as e:
        print(f"❌ Erreur lors du test: {e}")

if __name__ == "__main__":
    asyncio.run(test_full_negotiation())
