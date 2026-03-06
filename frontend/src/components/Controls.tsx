import { useState } from 'react'
import type { Participant, Room } from '../services/webrtc-manager'
import { 
  CameraIcon, MicrophoneIcon, VideoCameraIcon, StopScreenShareIcon, 
  VideoOffIcon, UserIcon, PhoneIcon 
} from '@heroicons/react/24/outline'

interface ControlsProps {
  localParticipant?: Participant
  room?: Room
  onToggleAudio: () => void
  onToggleVideo: () => void
  onStartScreenShare: () => void
  onStopScreenShare: () => void
  onLeaveRoom: () => void
  onToggleFullscreen: () => void
  isFullscreen: boolean
}

const Controls: React.FC<ControlsProps> = ({ 
  localParticipant, 
  room, 
  onToggleAudio, 
  onToggleVideo, 
  onStartScreenShare, 
  onStopScreenShare, 
  onLeaveRoom, 
  onToggleFullscreen, 
  isFullscreen 
}) => {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false)

  const getMainActionIcon = () => {
    if (localParticipant?.isScreenSharing) {
      return <StopScreenShareIcon className="h-5 w-5" />
    }
    if (!localParticipant?.isVideoMuted) {
      return <VideoCameraIcon className="h-5 w-5" />
    }
    if (!localParticipant?.isAudioMuted) {
      return <MicrophoneIcon className="h-5 w-5" />
    }
    return <PhoneIcon className="h-5 w-5" />
  }

  const getMainAction = () => {
    if (localParticipant?.isScreenSharing) {
      return onStopScreenShare
    }
    if (!localParticipant?.isVideoMuted) {
      return onToggleVideo
    }
    if (!localParticipant?.isAudioMuted) {
      return onToggleAudio
    }
    return () => {}
  }

  return (
    <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 z-50">
      <div className="bg-white bg-opacity-90 backdrop-blur-sm rounded-2xl shadow-2xl p-3 border border-gray-200">
        {/* Main Controls */}
        <div className="flex items-center justify-center gap-3 mb-3">
          <button
            onClick={onToggleAudio}
            className={`p-3 rounded-full transition-all duration-200 ${
              localParticipant?.isAudioMuted 
                ? 'bg-red-500 text-white' 
                : 'bg-gray-100 hover:bg-gray-200'
            }`}
          >
            <MicrophoneIcon className={`h-5 w-5 ${
              localParticipant?.isAudioMuted ? 'text-white' : 'text-gray-600'
            }`} />
          </button>

          <button
            onClick={onToggleVideo}
            className={`p-3 rounded-full transition-all duration-200 ${
              localParticipant?.isVideoMuted 
                ? 'bg-red-500 text-white' 
                : 'bg-gray-100 hover:bg-gray-200'
            }`}
          >
            <VideoCameraIcon className={`h-5 w-5 ${
              localParticipant?.isVideoMuted ? 'text-white' : 'text-gray-600'
            }`} />
          </button>

          <button
            onClick={getMainAction()}
            className={`p-3 rounded-full transition-all duration-200 ${
              localParticipant?.isScreenSharing || (!localParticipant?.isAudioMuted && !localParticipant?.isVideoMuted)
                ? 'bg-green-500 text-white' 
                : 'bg-gray-100 hover:bg-gray-200'
            }`}
          >
            {getMainActionIcon()}
          </button>

          {localParticipant && !localParticipant.isScreenSharing && (
            <button
              onClick={localParticipant?.isScreenSharing ? onStopScreenShare : onStartScreenShare}
              className={`p-3 rounded-full transition-all duration-200 ${
                localParticipant?.isScreenSharing 
                  ? 'bg-red-500 text-white' 
                  : 'bg-gray-100 hover:bg-gray-200'
              }`}
            >
              {localParticipant?.isScreenSharing ? (
                <StopScreenShareIcon className="h-5 w-5" />
              ) : (
                <ScreenShareIcon className="h-5 w-5" />
              )}
            </button>
          )}

          <button
            onClick={onLeaveRoom}
            className="p-3 rounded-full bg-red-500 text-white hover:bg-red-600 transition-colors"
          >
            <PhoneOffIcon className="h-5 w-5" />
          </button>
        </div>

        {/* Additional Controls */}
        <div className="flex items-center justify-center gap-3">
          <button
            onClick={onToggleFullscreen}
            className="p-2 rounded-lg bg-gray-100 hover:bg-gray-200 transition-colors"
          >
            {isFullscreen ? (
              <ExitFullscreenIcon className="h-4 w-4" />
            ) : (
              <FullscreenIcon className="h-4 w-4" />
            )}
          </button>

          <button
            onClick={() => setIsSettingsOpen(!isSettingsOpen)}
            className="p-2 rounded-lg bg-gray-100 hover:bg-gray-200 transition-colors relative"
          >
            <SettingsIcon className="h-4 w-4" />
            {isSettingsOpen && (
              <div className="absolute -bottom-10 -right-0 w-80 bg-white rounded-lg shadow-lg p-3">
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">Microphone</span>
                    <span className="text-sm font-medium">
                      {localParticipant?.isAudioMuted ? 'Muet' : 'Actif'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">Vidéo</span>
                    <span className="text-sm font-medium">
                      {localParticipant?.isVideoMuted ? 'Arrêtée' : 'Active'}
                    </span>
                  </div>
                  {room && (
                    <div className="flex items-center justify-between">
                      <span className="text-sm text-gray-600">Participants</span>
                      <span className="text-sm font-medium">{room.participants.length}</span>
                    </div>
                  )}
                </div>
              </div>
            )}
          </button>
        </div>
      </div>
    </div>
  )
}

export default Controls