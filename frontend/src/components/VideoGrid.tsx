import { useRef, useEffect } from 'react'
import type { Participant } from '../services/webrtc-manager'
import { UserIcon } from '@heroicons/react/24/outline'

interface VideoGridProps {
  participants: Participant[]
}

export const VideoGrid: React.FC<VideoGridProps> = ({ participants }) => {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 h-full overflow-y-auto p-4">
      {participants.map((participant) => (
        <VideoTile key={participant.id} participant={participant} />
      ))}
    </div>
  )
}

const VideoTile: React.FC<{ participant: Participant }> = ({ participant }) => {
  const videoRef = useRef<HTMLVideoElement>(null)

  useEffect(() => {
    if (videoRef.current && participant.videoTrack) {
      const stream = new MediaStream([participant.videoTrack])
      videoRef.current.srcObject = stream
    }
  }, [participant.videoTrack])

  return (
    <div className="relative aspect-video bg-gray-900 rounded-xl overflow-hidden shadow-lg border border-gray-800 group">
      {participant.videoTrack ? (
        <video
          ref={videoRef}
          autoPlay
          playsInline
          muted={participant.isLocal}
          className="w-full h-full object-cover"
        />
      ) : (
        <div className="w-full h-full flex flex-col items-center justify-center space-y-2">
          <div className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center">
            <UserIcon className="w-8 h-8 text-gray-400" />
          </div>
          <span className="text-gray-400 text-sm">{participant.name}</span>
        </div>
      )}
      
      <div className="absolute bottom-3 left-3 px-2 py-1 bg-black/50 backdrop-blur-md rounded text-white text-xs font-medium border border-white/10">
        {participant.name} {participant.isLocal ? '(Moi)' : ''}
      </div>
    </div>
  )
}
