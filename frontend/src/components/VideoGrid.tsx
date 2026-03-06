import { useState, useEffect, useRef } from 'react'
import type { Participant, Room } from '../services/webrtc-manager'
import { 
  CameraIcon, MicrophoneIcon, VideoCameraIcon, StopScreenShareIcon, 
  VideoOffIcon, UserIcon, PhoneIcon 
} from '@heroicons/react/24/outline'

interface VideoGridProps {
  participants: Participant[]
  localParticipant?: Participant
  isFullscreen?: boolean
}

const VideoGrid: React.FC<VideoGridProps> = ({ participants, localParticipant, isFullscreen = false }) => {
  const [gridLayout, setGridLayout] = useState<'grid' | 'speaker'>('grid')
  const videoRefs = useRef<Record<string, HTMLVideoElement | null>>({})

  useEffect(() => {
    // Auto-focus on active speaker (simple implementation)
    if (gridLayout === 'speaker' && participants.length > 0) {
      const activeParticipant = participants[0]
      const videoElement = videoRefs.current[activeParticipant.id]
      if (videoElement) {
        videoElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
      }
    }
  }, [participants, gridLayout])

  const getVideoTrack = (participant: Participant) => {
    if (participant.isScreenSharing && participant.screenTrack) {
      return participant.screenTrack
    }
    return participant.videoTrack
  }

  const renderVideo = (participant: Participant, index: number) => {
    const videoTrack = getVideoTrack(participant)
    const isLocal = participant.isLocal
    const isMuted = participant.isAudioMuted || participant.isVideoMuted
    const showPlaceholder = !videoTrack || !participant.stream

    return (
      <div
        key={participant.id}
        className={`relative ${isLocal ? 'z-10' : ''} ${showPlaceholder ? 'hidden' : ''}`}
      >
        <div
          className={`aspect-video bg-gray-900 rounded-lg overflow-hidden relative group ${isMuted ? 'opacity-60' : ''} ${isLocal ? 'border-2 border-blue-500' : ''}`}
        >
          {videoTrack && participant.stream && (
            <video
              ref={(el) => el ? videoRefs.current[participant.id] = el : null}
              autoPlay
              muted={isLocal}
              playsInline
              className="w-full h-full object-cover"
              srcObject={participant.stream}
            />
          )}
          
          {showPlaceholder && (
            <div className="w-full h-full flex items-center justify-center bg-gray-800">
              <div className="text-center">
                <UserAddIcon className="h-12 w-12 text-gray-400 mb-2" />
                <p className="text-sm text-gray-400">Aucune vidéo</p>
              </div>
            </div>
          )}

          <div className="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-20 transition-all duration-200 flex items-end">
            <div className="w-full p-3 flex items-center justify-between gap-2">
              <span className="text-white text-sm font-medium">
                {participant.name}
              </span>
              
              <div className="flex gap-1">
                {!participant.isAudioMuted && (
                  <MicrophoneIcon className="h-4 w-4 text-green-400" />
                )}
                {!participant.isVideoMuted && (
                  <VideoCameraIcon className="h-4 w-4 text-green-400" />
                )}
                {participant.isScreenSharing && (
                  <ScreenShareIcon className="h-4 w-4 text-blue-400" />
                )}
              </div>
            </div>
          </div>

          {isLocal && (
            <div className="absolute top-3 right-3 p-2 bg-white bg-opacity-80 backdrop-blur-sm rounded-full shadow-lg flex gap-1">
              <button
                onClick={() => console.log('Toggle microphone')}
                className="p-1 rounded hover:bg-gray-200 transition-colors"
              >
                {participant.isAudioMuted ? (
                  <MicOffIcon className="h-4 w-4 text-red-500" />
                ) : (
                  <MicrophoneIcon className="h-4 w-4 text-gray-600" />
                )}
              </button>
              <button
                onClick={() => console.log('Toggle video')}
                className="p-1 rounded hover:bg-gray-200 transition-colors"
              >
                {participant.isVideoMuted ? (
                  <VideoOffIcon className="h-4 w-4 text-red-500" />
                ) : (
                  <VideoCameraIcon className="h-4 w-4 text-gray-600" />
                )}
              </button>
              {participant.isScreenSharing ? (
                <button
                  onClick={() => console.log('Stop screen share')}
                  className="p-1 rounded hover:bg-gray-200 transition-colors"
                >
                  <StopScreenShareIcon className="h-4 w-4 text-gray-600" />
                </button>
              ) : (
                <button
                  onClick={() => console.log('Start screen share')}
                  className="p-1 rounded hover:bg-gray-200 transition-colors"
                >
                  <ScreenShareIcon className="h-4 w-4 text-gray-600" />
                </button>
              )}
            </div>
          )}
        </div>

        {index === 0 && gridLayout === 'speaker' && (
          <div className="absolute top-3 left-3 p-2 bg-white bg-opacity-80 backdrop-blur-sm rounded-full shadow-lg">
            <SpeakerIndicator />
          </div>
        )}
      </div>
    )
  }

  const getColumns = () => {
    if (isFullscreen) return 3
    if (participants.length <= 2) return 2
    if (participants.length <= 4) return 2
    if (participants.length <= 6) return 3
    return 4
  }

  const getVideoWidth = () => {
    const columns = getColumns()
    const width = `w-[${100 / columns}%]`
    return width
  }

  return (
    <div className={`relative ${isFullscreen ? 'h-screen' : 'h-96'}`}>
      <div className={`grid grid-cols-${getColumns()} gap-2 ${isFullscreen ? 'h-full' : ''}`}>
        {participants.map((participant, index) => (
          <div key={participant.id} className={getVideoWidth()}>
            {renderVideo(participant, index)}
          </div>
        ))}
      </div>

      {!isFullscreen && (
        <div className="absolute top-3 left-3 flex gap-2">
          <button
            onClick={() => setGridLayout(gl == 'grid' ? 'speaker' : 'grid')}
            className="p-2 bg-white bg-opacity-80 backdrop-blur-sm rounded-full shadow-lg hover:bg-opacity-90 transition-opacity"
          >
            {gridLayout === 'grid' ? (
              <FullscreenIcon className="h-5 w-5 text-gray-600" />
            ) : (
              <ExitFullscreenIcon className="h-5 w-5 text-gray-600" />
            )}
          </button>
        </div>
      )}
    </div>
  )
}

const SpeakerIndicator: React.FC = () => {
  return (
    <div className="flex items-center gap-1">
      <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
      <span className="text-xs text-red-500 font-medium">Actif</span>
    </div>
  )
}

export default VideoGrid