<template>
  <div class="room-detail">
    <div class="room-header">
      <h1>{{ room.name }}</h1>
      <el-tag :type="room.status === 'active' ? 'success' : 'info'">
        {{ room.status }}
      </el-tag>
    </div>
    
    <div class="room-info">
      <el-card>
        <div class="room-meta">
          <div class="meta-item">
            <span class="label">Room ID:</span>
            <span class="value">{{ room.id }}</span>
          </div>
          <div class="meta-item">
            <span class="label">Created:</span>
            <span class="value">{{ room.createdAt }}</span>
          </div>
          <div class="meta-item">
            <span class="label">Max Participants:</span>
            <span class="value">{{ room.maxParticipants }}</span>
          </div>
          <div class="meta-item">
            <span class="label">Participants:</span>
            <span class="value">{{ room.participants }}/{{ room.maxParticipants }}</span>
          </div>
        </div>
      </el-card>
    </div>
    
    <div class="room-content">
      <div class="video-section">
        <div class="local-video">
          <h3>Local Video</h3>
          <video ref="localVideo" autoplay muted></video>
          <div class="video-controls">
            <el-button @click="toggleCamera" :icon="isCameraEnabled ? 'el-icon-video-camera' : 'el-icon-video-camera-off'"></el-button>
            <el-button @click="toggleMicrophone" :icon="isMicrophoneEnabled ? 'el-icon-microphone' : 'el-icon-microphone-off'"></el-button>
            <el-button @click="toggleScreenShare" icon="el-icon-monitor"></el-button>
          </div>
        </div>
        
        <div class="remote-videos">
          <h3>Remote Videos</h3>
          <div class="video-grid">
            <video 
              v-for="remoteStream in remoteStreams" 
              :key="remoteStream.id"
              :ref="el => remoteVideoRefs[remoteStream.id] = el"
              :srcObject="remoteStream"
              autoplay
            ></video>
          </div>
        </div>
      </div>
      
      <div class="chat-section">
        <h3>Chat</h3>
        <div class="chat-container">
          <div class="messages" ref="messages">
            <div 
              v-for="message in messages" 
              :key="message.id"
              :class="['message', message.type]"
            >
              <div class="message-header">
                <span class="sender">{{ message.sender }}</span>
                <span class="timestamp">{{ formatTime(message.timestamp) }}</span>
              </div>
              <div class="message-content">
                {{ message.content }}
              </div>
            </div>
          </div>
          
          <div class="chat-input">
            <el-input
              v-model="chatInput"
              placeholder="Type a message..."
              @keyup.enter="sendMessage"
              :disabled="!isConnected"
            ></el-input>
            <el-button 
              @click="sendMessage" 
              :disabled="!isConnected || !chatInput.trim()"
              icon="el-icon-s-promotion"
            ></el-button>
          </div>
        </div>
      </div>
    </div>
    
    <div class="room-actions">
      <el-button type="primary" @click="startMeeting" v-if="room.status === 'inactive'">Start Meeting</el-button>
      <el-button type="warning" @click="endMeeting" v-if="room.status === 'active'">End Meeting</el-button>
      <el-button @click="leaveRoom">Leave Room</el-button>
      <el-button @click="manageRoom">Manage Room</el-button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, onUnmounted, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { useStore } from 'vuex'

export default {
  setup() {
    const router = useRouter()
    const store = useStore()
    
    const roomId = window.location.pathname.split('/').pop()
    
    const room = ref({
      id: roomId,
      name: 'Room Name',
      status: 'inactive',
      createdAt: '2024-01-01',
      maxParticipants: 10,
      participants: 0
    })
    
    const localVideo = ref(null)
    const messages = ref([])
    const chatInput = ref('')
    const isConnected = ref(false)
    const isCameraEnabled = ref(true)
    const isMicrophoneEnabled = ref(true)
    const remoteVideoRefs = reactive({})
    
    let localStream = null
    let peerConnection = null
    let dataChannel = null
    let signalingSocket = null
    let remoteStreams = ref([])
    
    const startMeeting = async () => {
      try {
        await getUserMedia()
        await initializePeerConnection()
        await connectToRoom()
        isConnected.value = true
        room.value.status = 'active'
      } catch (error) {
        console.error('Failed to start meeting:', error)
        alert('Failed to start meeting: ' + error.message)
      }
    }
    
    const getUserMedia = async () => {
      try {
        const constraints = {
          video: isCameraEnabled.value,
          audio: isMicrophoneEnabled.value
        }
        localStream = await navigator.mediaDevices.getUserMedia(constraints)
        localVideo.value.srcObject = localStream
      } catch (error) {
        console.error('Failed to get user media:', error)
        throw error
      }
    }
    
    const initializePeerConnection = () => {
      peerConnection = new RTCPeerConnection({
        iceServers: [
          { urls: 'stun:stun.l.google.com:19302' },
          { urls: 'stun:stun1.l.google.com:19302' }
        ]
      })
      
      peerConnection.onicecandidate = handleIceCandidate
      peerConnection.ontrack = handleRemoteTrack
      peerConnection.onconnectionstatechange = handleConnectionStateChange
      
      localStream.getTracks().forEach(track => {
        peerConnection.addTrack(track, localStream)
      })
      
      dataChannel = peerConnection.createDataChannel('chat')
      dataChannel.onmessage = handleDataChannelMessage
      dataChannel.onopen = handleDataChannelOpen
      dataChannel.onclose = handleDataChannelClose
    }
    
    const connectToRoom = async () => {
      try {
        const offer = await peerConnection.createOffer()
        await peerConnection.setLocalDescription(offer)
        
        // TODO: Send offer to signaling server
        console.log('Sending offer to signaling server:', offer)
        
      } catch (error) {
        console.error('Failed to connect to room:', error)
        throw error
      }
    }
    
    const handleIceCandidate = (event) => {
      if (event.candidate) {
        // TODO: Send ICE candidate to signaling server
        console.log('Sending ICE candidate:', event.candidate)
      }
    }
    
    const handleRemoteTrack = (event) => {
      const stream = event.streams[0]
      remoteStreams.value.push(stream)
      
      // Force re-render to show new video element
      remoteStreams.value = [...remoteStreams.value]
    }
    
    const handleConnectionStateChange = () => {
      console.log('Connection state:', peerConnection.connectionState)
    }
    
    const handleDataChannelMessage = (event) => {
      const message = JSON.parse(event.data)
      messages.value.push(message)
      
      // Scroll to bottom
      setTimeout(() => {
        const messagesEl = this.$refs.messages
        if (messagesEl) {
          messagesEl.scrollTop = messagesEl.scrollHeight
        }
      }, 0)
    }
    
    const handleDataChannelOpen = () => {
      console.log('Data channel opened')
    }
    
    const handleDataChannelClose = () => {
      console.log('Data channel closed')
    }
    
    const sendMessage = () => {
      if (chatInput.value.trim() && dataChannel && dataChannel.readyState === 'open') {
        const message = {
          id: Date.now(),
          sender: 'Me',
          content: chatInput.value,
          timestamp: new Date(),
          type: 'sent'
        }
        
        dataChannel.send(JSON.stringify(message))
        messages.value.push(message)
        chatInput.value = ''
        
        // Scroll to bottom
        setTimeout(() => {
          const messagesEl = this.$refs.messages
          if (messagesEl) {
            messagesEl.scrollTop = messagesEl.scrollHeight
          }
        }, 0)
      }
    }
    
    const toggleCamera = () => {
      isCameraEnabled.value = !isCameraEnabled.value
      updateMediaConstraints()
    }
    
    const toggleMicrophone = () => {
      isMicrophoneEnabled.value = !isMicrophoneEnabled.value
      updateMediaConstraints()
    }
    
    const toggleScreenShare = async () => {
      try {
        const screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true })
        const videoTrack = screenStream.getVideoTracks()[0]
        
        localStream.removeTrack(localStream.getVideoTracks()[0])
        localStream.addTrack(videoTrack)
        
        // Replace video track in peer connection
        peerConnection.getSenders().forEach(sender => {
          if (sender.track && sender.track.kind === 'video') {
            sender.replaceTrack(videoTrack)
          }
        })
        
        // Update local video element
        localVideo.value.srcObject = localStream
      } catch (error) {
        console.error('Failed to share screen:', error)
      }
    }
    
    const updateMediaConstraints = async () => {
      try {
        const constraints = {
          video: isCameraEnabled.value,
          audio: isMicrophoneEnabled.value
        }
        
        const newStream = await navigator.mediaDevices.getUserMedia(constraints)
        
        // Replace tracks in peer connection
        localStream.getTracks().forEach(track => {
          peerConnection.removeTrack(peerConnection.addTrack(track, newStream))
        })
        
        // Update local video
        localVideo.value.srcObject = newStream
        localStream = newStream
      } catch (error) {
        console.error('Failed to update media constraints:', error)
      }
    }
    
    const formatTime = (timestamp) => {
      return new Date(timestamp).toLocaleTimeString()
    }
    
    const leaveRoom = () => {
      if (peerConnection) {
        peerConnection.close()
      }
      if (signalingSocket) {
        signalingSocket.close()
      }
      remoteStreams.value = []
      isConnected.value = false
      router.push('/rooms')
    }
    
    const endMeeting = () => {
      // TODO: Implement end meeting logic
      console.log('Ending meeting...')
      leaveRoom()
      room.value.status = 'inactive'
    }
    
    const manageRoom = () => {
      router.push(`/room/${roomId}/manage`)
    }
    
    onUnmounted(() => {
      leaveRoom()
    })
    
    return {
      room,
      localVideo,
      messages,
      chatInput,
      isConnected,
      isCameraEnabled,
      isMicrophoneEnabled,
      remoteVideoRefs,
      startMeeting,
      sendMessage,
      toggleCamera,
      toggleMicrophone,
      toggleScreenShare,
      leaveRoom,
      endMeeting,
      manageRoom,
      formatTime
    }
  }
}
</script>

<style scoped>
.room-detail {
  padding: 20px;
}

.room-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.room-header h1 {
  color: #333;
}

.room-info {
  margin-bottom: 30px;
}

.room-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
}

.meta-item {
  display: flex;
  flex-direction: column;
}

.label {
  font-size: 12px;
  color: #666;
  margin-bottom: 5px;
}

.value {
  font-weight: bold;
  color: #333;
}

.room-content {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 20px;
  margin-bottom: 30px;
}

.video-section h3 {
  margin-bottom: 15px;
  color: #333;
}

.local-video {
  margin-bottom: 20px;
}

.local-video video {
  width: 100%;
  height: 200px;
  object-fit: cover;
  border-radius: 8px;
  margin-bottom: 10px;
}

.video-controls {
  display: flex;
  gap: 10px;
}

.video-controls .el-button {
  padding: 8px 12px;
}

.remote-videos h3 {
  margin-bottom: 15px;
  color: #333;
}

.video-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 15px;
}

.video-grid video {
  width: 100%;
  height: 150px;
  object-fit: cover;
  border-radius: 8px;
}

.chat-section h3 {
  margin-bottom: 15px;
  color: #333;
}

.chat-container {
  display: flex;
  flex-direction: column;
  height: 400px;
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 15px;
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  background: #f5f7fa;
}

.message {
  margin-bottom: 15px;
}

.message.sent {
  text-align: right;
}

.message.received {
  text-align: left;
}

.message-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
  font-size: 12px;
  color: #666;
}

.message-content {
  display: inline-block;
  padding: 8px 12px;
  border-radius: 18px;
  max-width: 70%;
  word-wrap: break-word;
}

.message.sent .message-content {
  background: #409eff;
  color: white;
  text-align: left;
}

.message.received .message-content {
  background: white;
  color: #333;
  border: 1px solid #e4e7ed;
}

.chat-input {
  display: flex;
  gap: 10px;
  margin-top: 10px;
}

.chat-input .el-input {
  flex: 1;
}

.room-actions {
  display: flex;
  gap: 10px;
  justify-content: center;
}
</style>