import { Peer } from 'peerjs'
import io from 'socket.io-client'
import { v4 as uuidv4 } from 'uuid'

export interface Participant {
  id: string
  name: string
  videoTrack?: MediaStreamTrack
  audioTrack?: MediaStreamTrack
  screenTrack?: MediaStreamTrack
  isLocal: boolean
  isAudioMuted: boolean
  isVideoMuted: boolean
  isScreenSharing: boolean
}

export interface Room {
  id: string
  name: string
  participants: Participant[]
  isLocalParticipant?: boolean
}

export interface SignalingMessage {
  type: 'offer' | 'answer' | 'candidate' | 'join' | 'leave' | 'update'
  senderId: string
  receiverId?: string
  roomId: string
  data: any
}

export class WebRTCManager {
  private peer: Peer
  private socket: SocketIOClient.Socket
  private localStream: MediaStream | null = null
  private participants: Map<string, Participant> = new Map()
  private room: Room | null = null
  private iceServers: RTCIceServer[] = [
    {
      urls: [
        'stun:stun.l.google.com:19302',
        'stun:stun1.l.google.com:19302',
        'stun:stun2.l.google.com:19302',
        'stun:stun3.l.google.com:19302',
        'stun:stun4.l.google.com:19302'
      ]
    },
    {
      urls: 'turn:turn-server.example.com:3478',
      username: 'username',
      credential: 'password'
    }
  ]

  constructor() {
    this.initializePeer()
    this.initializeSocket()
  }

  private initializePeer(): void {
    this.peer = new Peer({
      host: window.location.hostname,
      port: 8080,
      path: '/peerjs',
      secure: window.location.protocol === 'https:',
      config: {
        iceServers: this.iceServers
      }
    })

    this.peer.on('open', (id) => {
      console.log('Peer connection opened with ID:', id)
    })

    this.peer.on('connection', (conn) => {
      this.handlePeerConnection(conn)
    })

    this.peer.on('call', (call) => {
      this.handleIncomingCall(call)
    })

    this.peer.on('error', (err) => {
      console.error('Peer error:', err)
    })
  }

  private initializeSocket(): void {
    this.socket = io('/room', {
      path: '/socket.io',
      transports: ['websocket'],
      autoConnect: false
    })

    this.socket.on('connect', () => {
      console.log('Socket connected')
    })

    this.socket.on('disconnect', () => {
      console.log('Socket disconnected')
    })

    this.socket.on('signaling', (message: SignalingMessage) => {
      this.handleSignalingMessage(message)
    })

    this.socket.on('room-update', (room: Room) => {
      this.updateRoom(room)
    })

    this.socket.on('error', (error: any) => {
      console.error('Socket error:', error)
    })
  }

  async joinRoom(roomId: string, userName: string): Promise<void> {
    try {
      // Get user media
      this.localStream = await this.getUserMedia()
      
      // Connect to socket
      await this.connectSocket()
      
      // Join room
      this.socket.emit('join-room', {
        roomId,
        userId: this.peer.id,
        userName
      })

      // Create local participant
      const localParticipant: Participant = {
        id: this.peer.id,
        name: userName,
        videoTrack: this.localStream.getVideoTracks()[0],
        audioTrack: this.localStream.getAudioTracks()[0],
        isLocal: true,
        isAudioMuted: false,
        isVideoMuted: false,
        isScreenSharing: false
      }

      this.participants.set(this.peer.id, localParticipant)

      // Update room state
      this.room = {
        id: roomId,
        name: `Room ${roomId}`,
        participants: Array.from(this.participants.values()),
        isLocalParticipant: true
      }

      console.log('Joined room:', roomId)
    } catch (error) {
      console.error('Failed to join room:', error)
      throw error
    }
  }

  async leaveRoom(): Promise<void> {
    try {
      if (this.room) {
        this.socket.emit('leave-room', {
          roomId: this.room.id,
          userId: this.peer.id
        })
      }

      // Close all peer connections
      this.participants.forEach((participant) => {
        if (participant.videoTrack) {
          participant.videoTrack.stop()
        }
        if (participant.audioTrack) {
          participant.audioTrack.stop()
        }
      })

      // Close socket
      this.socket.disconnect()

      // Reset state
      this.participants.clear()
      this.room = null
      this.localStream = null

      console.log('Left room')
    } catch (error) {
      console.error('Failed to leave room:', error)
      throw error
    }
  }

  async startScreenShare(): Promise<void> {
    try {
      const screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: {
          cursor: 'always'
        },
        audio: false
      })

      const screenTrack = screenStream.getVideoTracks()[0]
      
      // Update local participant
      const localParticipant = this.participants.get(this.peer.id)
      if (localParticipant) {
        localParticipant.screenTrack = screenTrack
        localParticipant.isScreenSharing = true
        this.participants.set(this.peer.id, localParticipant)

        // Share screen with other participants
        this.participants.forEach((participant, id) => {
          if (id !== this.peer.id && participant.videoTrack) {
            this.shareScreenWithParticipant(participant.id, screenTrack)
          }
        })
      }

      console.log('Screen sharing started')
    } catch (error) {
      console.error('Failed to start screen share:', error)
      throw error
    }
  }

  async stopScreenShare(): Promise<void> {
    try {
      const localParticipant = this.participants.get(this.peer.id)
      if (localParticipant && localParticipant.screenTrack) {
        localParticipant.screenTrack.stop()
        localParticipant.screenTrack = undefined
        localParticipant.isScreenSharing = false
        this.participants.set(this.peer.id, localParticipant)

        console.log('Screen sharing stopped')
      }
    } catch (error) {
      console.error('Failed to stop screen share:', error)
      throw error
    }
  }

  async muteAudio(): Promise<void> {
    try {
      const localParticipant = this.participants.get(this.peer.id)
      if (localParticipant && localParticipant.audioTrack) {
        localParticipant.audioTrack.enabled = false
        localParticipant.isAudioMuted = true
        this.participants.set(this.peer.id, localParticipant)

        console.log('Audio muted')
      }
    } catch (error) {
      console.error('Failed to mute audio:', error)
      throw error
    }
  }

  async unmuteAudio(): Promise<void> {
    try {
      const localParticipant = this.participants.get(this.peer.id)
      if (localParticipant && localParticipant.audioTrack) {
        localParticipant.audioTrack.enabled = true
        localParticipant.isAudioMuted = false
        this.participants.set(this.peer.id, localParticipant)

        console.log('Audio unmuted')
      }
    } catch (error) {
      console.error('Failed to unmute audio:', error)
      throw error
    }
  }

  async muteVideo(): Promise<void> {
    try {
      const localParticipant = this.participants.get(this.peer.id)
      if (localParticipant && localParticipant.videoTrack) {
        localParticipant.videoTrack.enabled = false
        localParticipant.isVideoMuted = true
        this.participants.set(this.peer.id, localParticipant)

        console.log('Video muted')
      }
    } catch (error) {
      console.error('Failed to mute video:', error)
      throw error
    }
  }

  async unmuteVideo(): Promise<void> {
    try {
      const localParticipant = this.participants.get(this.peer.id)
      if (localParticipant && localParticipant.videoTrack) {
        localParticipant.videoTrack.enabled = true
        localParticipant.isVideoMuted = false
        this.participants.set(this.peer.id, localParticipant)

        console.log('Video unmuted')
      }
    } catch (error) {
      console.error('Failed to unmute video:', error)
      throw error
    }
  }

  private async getUserMedia(): Promise<MediaStream> {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: {
          width: { ideal: 1280 },
          height: { ideal: 720 },
          frameRate: { ideal: 30 }
        },
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true
        }
      })

      return stream
    } catch (error) {
      console.error('Failed to get user media:', error)
      throw error
    }
  }

  private async connectSocket(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.socket.once('connect', resolve)
      this.socket.once('connect_error', reject)
      this.socket.connect()
    })
  }

  private handlePeerConnection(conn: Peer.DataConnection): void {
    conn.on('data', (data) => {
      console.log('Received peer data:', data)
    })

    conn.on('open', () => {
      console.log('Peer connection opened')
    })

    conn.on('close', () => {
      console.log('Peer connection closed')
    })

    conn.on('error', (err) => {
      console.error('Peer connection error:', err)
    })
  }

  private handleIncomingCall(call: Peer.MediaConnection): void {
    if (this.localStream) {
      call.answer(this.localStream)
    }

    call.on('stream', (remoteStream) => {
      console.log('Received remote stream')
      
      // Create participant for remote user
      const remoteParticipant: Participant = {
        id: call.peer,
        name: `User ${call.peer}`,
        videoTrack: remoteStream.getVideoTracks()[0],
        audioTrack: remoteStream.getAudioTracks()[0],
        isLocal: false,
        isAudioMuted: false,
        isVideoMuted: false,
        isScreenSharing: false
      }

      this.participants.set(call.peer, remoteParticipant)
      
      // Emit room update
      if (this.room) {
        this.room.participants = Array.from(this.participants.values())
        this.socket.emit('room-update', this.room)
      }
    })

    call.on('close', () => {
      console.log('Call closed')
      
      // Remove participant
      this.participants.delete(call.peer)
      
      // Emit room update
      if (this.room) {
        this.room.participants = Array.from(this.participants.values())
        this.socket.emit('room-update', this.room)
      }
    })

    call.on('error', (err) => {
      console.error('Call error:', err)
    })
  }

  private handleSignalingMessage(message: SignalingMessage): void {
    switch (message.type) {
      case 'offer':
        this.handleOffer(message)
        break
      case 'answer':
        this.handleAnswer(message)
        break
      case 'candidate':
        this.handleCandidate(message)
        break
      case 'join':
        this.handleParticipantJoin(message)
        break
      case 'leave':
        this.handleParticipantLeave(message)
        break
      case 'update':
        this.handleRoomUpdate(message)
        break
    }
  }

  private handleOffer(message: SignalingMessage): void {
    // Handle offer from remote peer
    console.log('Received offer from:', message.senderId)
    
    // Create answer and send back
    if (this.localStream) {
      const call = this.peer.call(message.senderId, this.localStream)
      
      call.on('stream', (remoteStream) => {
        console.log('Received remote stream in answer')
        
        // Create participant for remote user
        const remoteParticipant: Participant = {
          id: message.senderId,
          name: `User ${message.senderId}`,
          videoTrack: remoteStream.getVideoTracks()[0],
          audioTrack: remoteStream.getAudioTracks()[0],
          isLocal: false,
          isAudioMuted: false,
          isVideoMuted: false,
          isScreenSharing: false
        }

        this.participants.set(message.senderId, remoteParticipant)
        
        // Emit room update
        if (this.room) {
          this.room.participants = Array.from(this.participants.values())
          this.socket.emit('room-update', this.room)
        }
      })
    }
  }

  private handleAnswer(message: SignalingMessage): void {
    console.log('Received answer from:', message.senderId)
  }

  private handleCandidate(message: SignalingMessage): void {
    console.log('Received candidate from:', message.senderId)
  }

  private handleParticipantJoin(message: SignalingMessage): void {
    console.log('Participant joined:', message.senderId)
    
    // Create participant
    const newParticipant: Participant = {
      id: message.senderId,
      name: message.data?.userName || `User ${message.senderId}`,
      isLocal: false,
      isAudioMuted: false,
      isVideoMuted: false,
      isScreenSharing: false
    }

    this.participants.set(message.senderId, newParticipant)
    
    // Emit room update
    if (this.room) {
      this.room.participants = Array.from(this.participants.values())
      this.socket.emit('room-update', this.room)
    }
  }

  private handleParticipantLeave(message: SignalingMessage): void {
    console.log('Participant left:', message.senderId)
    
    // Remove participant
    this.participants.delete(message.senderId)
    
    // Emit room update
    if (this.room) {
      this.room.participants = Array.from(this.participants.values())
      this.socket.emit('room-update', this.room)
    }
  }

  private handleRoomUpdate(message: SignalingMessage): void {
    console.log('Room updated:', message.data)
    
    if (message.data && message.data.room) {
      this.room = message.data.room
    }
  }

  private shareScreenWithParticipant(participantId: string, screenTrack: MediaStreamTrack): void {
    // This would involve creating a new peer connection specifically for screen sharing
    // For simplicity, we're not implementing full screen sharing here
    console.log('Sharing screen with participant:', participantId)
  }

  getParticipants(): Participant[] {
    return Array.from(this.participants.values())
  }

  getLocalParticipant(): Participant | undefined {
    return this.participants.get(this.peer.id)
  }

  getRoom(): Room | null {
    return this.room
  }

  isConnected(): boolean {
    return this.socket.connected && this.peer.open
  }
}