import { useState, useEffect, useMemo } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { WebRTCManager } from '../services/webrtc-manager'
import type { Participant } from '../services/webrtc-manager'
import { VideoGrid } from '../components/VideoGrid'
import { Controls } from '../components/Controls'

export const MeetingRoom = () => {
  const { roomId } = useParams<{ roomId: string }>()
  const navigate = useNavigate()
  const [participants, setParticipants] = useState<Participant[]>([])
  const [mode, setMode] = useState<'publisher' | 'viewer' | null>(null)
  const [userName] = useState(`User_${Math.floor(Math.random() * 1000)}`)

  // Initialisation du manager avec le callback de mise à jour
  const webrtc = useMemo(() => new WebRTCManager((p) => {
    setParticipants([...p]); // Force un nouveau tableau pour le re-render
  }), []);

  useEffect(() => {
    if (!roomId || !mode) return;

    const start = async () => {
      try {
        await webrtc.joinRoom(roomId, userName, mode);
      } catch (err) {
        console.error("Failed to join:", err);
      }
    };

    start();
  }, [roomId, mode, userName, webrtc]);

  if (!mode) {
    return (
      <div className="h-screen flex items-center justify-center bg-gray-900">
        <div className="bg-gray-800 p-8 rounded-2xl shadow-2xl text-center space-y-6 border border-gray-700">
          <h1 className="text-2xl font-bold text-white">Multi-Meet DRMP</h1>
          <p className="text-gray-400">ID Salle: {roomId}</p>
          <div className="flex gap-4">
            <button 
              onClick={() => setMode('publisher')}
              className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-xl transition-all font-bold"
            >
              Rejoindre et Diffuser
            </button>
            <button 
              onClick={() => setMode('viewer')}
              className="px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white rounded-xl transition-all font-bold"
            >
              Rejoindre en Spectateur
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-screen bg-black">
      <div className="flex-1 p-4 overflow-hidden relative">
        <VideoGrid participants={participants} />
        {participants.length === 1 && (
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
            <p className="text-gray-500 bg-black/20 px-4 py-2 rounded-full backdrop-blur-sm">
              En attente d'autres participants...
            </p>
          </div>
        )}
      </div>
      <div className="h-20 bg-gray-900 border-t border-gray-800 flex items-center justify-center px-4">
        <Controls 
          onLeave={() => window.location.href = '/'} 
          onMuteAudio={() => {}}
          onMuteVideo={() => {}}
          isAudioMuted={false}
          isVideoMuted={mode === 'viewer'}
        />
        <div className="ml-auto flex items-center gap-3">
          <div className="flex flex-col items-end">
            <span className="text-white text-sm font-medium">{userName}</span>
            <span className="text-gray-500 text-xs">{mode === 'publisher' ? 'Diffuseur' : 'Spectateur'}</span>
          </div>
          <div className="w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white font-bold">
            {userName.charAt(0)}
          </div>
        </div>
      </div>
    </div>
  )
}

export default MeetingRoom;
