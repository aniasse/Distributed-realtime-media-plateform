import { MicrophoneIcon, VideoCameraIcon, PhoneXMarkIcon } from '@heroicons/react/24/outline'

interface ControlsProps {
  onLeave: () => void
  onMuteAudio: () => void
  onMuteVideo: () => void
  isAudioMuted: boolean
  isVideoMuted: boolean
}

export const Controls: React.FC<ControlsProps> = ({ 
  onLeave, 
  onMuteAudio, 
  onMuteVideo, 
  isAudioMuted, 
  isVideoMuted 
}) => {
  return (
    <div className="flex items-center space-x-4">
      <button
        onClick={onMuteAudio}
        className={`p-3 rounded-full ${isAudioMuted ? 'bg-red-500' : 'bg-gray-200'} hover:opacity-80 transition-all`}
      >
        <MicrophoneIcon className="h-6 w-6 text-gray-800" />
      </button>

      <button
        onClick={onMuteVideo}
        className={`p-3 rounded-full ${isVideoMuted ? 'bg-red-500' : 'bg-gray-200'} hover:opacity-80 transition-all`}
      >
        <VideoCameraIcon className="h-6 w-6 text-gray-800" />
      </button>

      <button
        onClick={onLeave}
        className="p-3 rounded-full bg-red-600 hover:bg-red-700 transition-all"
      >
        <PhoneXMarkIcon className="h-6 w-6 text-white" />
      </button>
    </div>
  )
}
