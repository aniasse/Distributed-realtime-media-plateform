import { useState, useEffect } from 'react'
import { WebRTCManager, Participant, Room } from '@/services/webrtc-manager'

interface UseWebRTCReturn {
  webrtcManager: WebRTCManager
  room: Room | null
  participants: Participant[]
  localParticipant: Participant | undefined
  isConnected: boolean
  joinRoom: (roomId: string, userName: string) => Promise<void>
  leaveRoom: () => Promise<void>
  startScreenShare: () => Promise<void>
  stopScreenShare: () => Promise<void>
  muteAudio: () => Promise<void>
  unmuteAudio: () => Promise<void>
  muteVideo: () => Promise<void>
  unmuteVideo: () => Promise<void>
}

export function useWebRTC(): UseWebRTCReturn {
  const [webrtcManager] = useState(() => new WebRTCManager())
  const [room, setRoom] = useState<Room | null>(null)
  const [participants, setParticipants] = useState<Participant[]>([])
  const [localParticipant, setLocalParticipant] = useState<Participant | undefined>()
  const [isConnected, setIsConnected] = useState(false)

  useEffect(() => {
    const handleRoomUpdate = (updatedRoom: Room) => {
      setRoom(updatedRoom)
      setParticipants(updatedRoom.participants)
      
      const local = updatedRoom.participants.find(p => p.isLocal)
      if (local) {
        setLocalParticipant(local)
      }
    }

    const handleConnectionStatus = (connected: boolean) => {
      setIsConnected(connected)
    }

    webrtcManager.on('room-update', handleRoomUpdate)
    webrtcManager.on('connection-status', handleConnectionStatus)

    return () => {
      webrtcManager.off('room-update', handleRoomUpdate)
      webrtcManager.off('connection-status', handleConnectionStatus)
    }
  }, [webrtcManager])

  const joinRoom = async (roomId: string, userName: string): Promise<void> => {
    try {
      await webrtcManager.joinRoom(roomId, userName)
    } catch (error) {
      console.error('Failed to join room:', error)
      throw error
    }
  }

  const leaveRoom = async (): Promise<void> => {
    try {
      await webrtcManager.leaveRoom()
    } catch (error) {
      console.error('Failed to leave room:', error)
      throw error
    }
  }

  const startScreenShare = async (): Promise<void> => {
    try {
      await webrtcManager.startScreenShare()
    } catch (error) {
      console.error('Failed to start screen share:', error)
      throw error
    }
  }

  const stopScreenShare = async (): Promise<void> => {
    try {
      await webrtcManager.stopScreenShare()
    } catch (error) {
      console.error('Failed to stop screen share:', error)
      throw error
    }
  }

  const muteAudio = async (): Promise<void> => {
    try {
      await webrtcManager.muteAudio()
    } catch (error) {
      console.error('Failed to mute audio:', error)
      throw error
    }
  }

  const unmuteAudio = async (): Promise<void> => {
    try {
      await webrtcManager.unmuteAudio()
    } catch (error) {
      console.error('Failed to unmute audio:', error)
      throw error
    }
  }

  const muteVideo = async (): Promise<void> => {
    try {
      await webrtcManager.muteVideo()
    } catch (error) {
      console.error('Failed to mute video:', error)
      throw error
    }
  }

  const unmuteVideo = async (): Promise<void> => {
    try {
      await webrtcManager.unmuteVideo()
    } catch (error) {
      console.error('Failed to unmute video:', error)
      throw error
    }
  }

  return {
    webrtcManager,
    room,
    participants,
    localParticipant,
    isConnected,
    joinRoom,
    leaveRoom,
    startScreenShare,
    stopScreenShare,
    muteAudio,
    unmuteAudio,
    muteVideo,
    unmuteVideo
  }
}