import { useState, useEffect } from 'react'
import type { Participant, Room } from '../services/webrtc-manager'
import { VideoGrid } from '../components/VideoGrid'
import { Controls } from '../components/Controls'
import { useNavigate, useParams } from 'react-router-dom'

const MeetingRoom: React.FC = () => {
  const [room, setRoom] = useState<Room | null>(null)
  const [localParticipant, setLocalParticipant] = useState<Participant | undefined>()
  const [participants, setParticipants] = useState<Participant[]>([])
  const [isFullscreen, setIsFullscreen] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [roomId] = useState<string>('123456')
  const [userName] = useState<string>('Aniasse')

  const navigate = useNavigate()

  useEffect(() => {
    const initializeMeeting = async () => {
      try {
        const { WebRTCManager } = await import('../services/webrtc-manager')
        const webrtcManager = new WebRTCManager()

        // Join room
        await webrtcManager.joinRoom(roomId, userName)

        // Set up event listeners
        webrtcManager.on('room-update', (updatedRoom: Room) => {
          setRoom(updatedRoom)
          setParticipants(updatedRoom.participants)
        })

        webrtcManager.on('participant-joined', (participant: Participant) => {
          setParticipants(prev => [...prev, participant])
        })

        webrtcManager.on('participant-left', (participantId: string) => {
          setParticipants(prev => prev.filter(p => p.id !== participantId))
        })

        webrtcManager.on('local-participant-updated', (participant: Participant) => {
          setLocalParticipant(participant)
        })

        // Get initial state
        const initialRoom = webrtcManager.getRoom()
        const initialLocalParticipant = webrtcManager.getLocalParticipant()

        setRoom(initialRoom)
        setLocalParticipant(initialLocalParticipant)
        setParticipants(initialRoom?.participants || [])

        setIsLoading(false)
      } catch (error) {
        console.error('Failed to initialize meeting:', error)
        setIsLoading(false)
      }
    }

    initializeMeeting()

    // Cleanup
    return () => {
      if (localParticipant) {
        // Leave room
      }
    }
  }, [roomId, userName, localParticipant])

  const handleToggleAudio = () => {
    console.log('Toggle audio')
    // webrtcManager.muteAudio()
  }

  const handleToggleVideo = () => {
    console.log('Toggle video')
    // webrtcManager.muteVideo()
  }

  const handleStartScreenShare = () => {
    console.log('Start screen share')
    // webrtcManager.startScreenShare()
  }

  const handleStopScreenShare = () => {
    console.log('Stop screen share')
    // webrtcManager.stopScreenShare()
  }

  const handleLeaveRoom = () => {
    navigate('/lobby')
  }

  const handleToggleFullscreen = () => {
    setIsFullscreen(!isFullscreen)
    if (!isFullscreen) {
      document.documentElement.requestFullscreen()
    } else {
      document.exitFullscreen()
    }
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900">
        <div className="text-white text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p>Connexion à la salle...</p>
        </div>
      </div>
    )
  }

  if (!room) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900">
        <div className="text-white text-center">
          <p>Impossible de se connecter à la salle</p>
        </div>
      </div>
    )
  }

  return (
    <div className="relative h-screen bg-gray-900">
      {/* Video Grid */}
      <VideoGrid 
        participants={participants}
        localParticipant={localParticipant}
        isFullscreen={isFullscreen}
      />

      {/* Controls */}
      <Controls
        localParticipant={localParticipant}
        room={room}
        onToggleAudio={handleToggleAudio}
        onToggleVideo={handleToggleVideo}
        onStartScreenShare={handleStartScreenShare}
        onStopScreenShare={handleStopScreenShare}
        onLeaveRoom={handleLeaveRoom}
        onToggleFullscreen={handleToggleFullscreen}
        isFullscreen={isFullscreen}
      />

      {/* Room Info */}
      <div className="absolute top-4 left-4 bg-white bg-opacity-80 backdrop-blur-sm rounded-lg p-3">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
          <span className="text-sm font-medium text-gray-900">Salle {room.id}</span>
          <span className="text-xs text-gray-500">{participants.length} participants</span>
        </div>
      </div>
    </div>
  )
}

export default MeetingRoom