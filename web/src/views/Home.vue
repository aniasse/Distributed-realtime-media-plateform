<template>
  <div class="home">
    <div class="hero">
      <h1>DRMP - Distributed Realtime Media Platform</h1>
      <p>Professional live streaming and online meeting solution</p>
      <div class="buttons">
        <el-button type="primary" @click="startStreaming" v-if="isLoggedIn">Start Streaming</el-button>
        <el-button type="success" @click="createRoom" v-if="isLoggedIn">Create Room</el-button>
        <el-button type="info" @click="goToLogin" v-if="!isLoggedIn">Login</el-button>
      </div>
    </div>
    
    <div class="features">
      <h2>Features</h2>
      <el-row :gutter="20">
        <el-col :span="6">
          <div class="feature-card">
            <i class="el-icon-video-camera"></i>
            <h3>Live Streaming</h3>
            <p>RTMP and WebRTC streaming support</p>
          </div>
        </el-col>
        <el-col :span="6">
          <div class="feature-card">
            <i class="el-icon-user"></i>
            <h3>Online Meetings</h3>
            <p>Real-time video conferencing</p>
          </div>
        </el-col>
        <el-col :span="6">
          <div class="feature-card">
            <i class="el-icon-video-play"></i>
            <h3>Recording</h3>
            <p>Automatic session recording</p>
          </div>
        </el-col>
        <el-col :span="6">
          <div class="feature-card">
            <i class="el-icon-s-data"></i>
            <h3>Analytics</h3>
            <p>Real-time performance monitoring</p>
          </div>
        </el-col>
      </el-row>
    </div>
  </div>
</template>

<script>
import { computed } from 'vue'
import { useStore } from 'vuex'

export default {
  setup() {
    const store = useStore()

    const isLoggedIn = computed(() => store.getters.isLoggedIn)

    const startStreaming = () => {
      router.push('/dashboard')
    }

    const createRoom = () => {
      router.push('/rooms')
    }

    const goToLogin = () => {
      router.push('/login')
    }

    return {
      isLoggedIn,
      startStreaming,
      createRoom,
      goToLogin
    }
  }
}
</script>

<style scoped>
.hero {
  text-align: center;
  padding: 60px 0;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.hero h1 {
  font-size: 3.5rem;
  margin-bottom: 20px;
}

.hero p {
  font-size: 1.5rem;
  margin-bottom: 30px;
}

.buttons {
  display: flex;
  justify-content: center;
  gap: 20px;
}

.features {
  padding: 60px 0;
  text-align: center;
}

.feature-card {
  padding: 30px;
  background: #f8f9fa;
  border-radius: 10px;
  transition: transform 0.3s;
  cursor: pointer;
}

.feature-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 30px rgba(0,0,0,0.1);
}

.feature-card i {
  font-size: 3rem;
  color: #667eea;
  margin-bottom: 15px;
}

.feature-card h3 {
  margin-bottom: 10px;
  color: #333;
}

.feature-card p {
  color: #666;
}
</style>