<template>
  <div class="dashboard">
    <div class="dashboard-header">
      <h1>Dashboard</h1>
      <el-button type="primary" @click="createRoom">Create New Room</el-button>
    </div>
    
    <div class="stats-grid">
      <el-card class="stat-card">
        <h3>Active Rooms</h3>
        <p class="stat-number">{{ stats.activeRooms || 0 }}</p>
      </el-card>
      <el-card class="stat-card">
        <h3>Total Users</h3>
        <p class="stat-number">{{ stats.totalUsers || 0 }}</p>
      </el-card>
      <el-card class="stat-card">
        <h3>Stream Quality</h3>
        <p class="stat-number">HD</p>
      </el-card>
      <el-card class="stat-card">
        <h3>Latency</h3>
        <p class="stat-number">{{ stats.latency || '0ms' }}</p>
      </el-card>
    </div>
    
    <div class="rooms-section">
      <h2>Your Rooms</h2>
      <el-table :data="rooms" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80"></el-table-column>
        <el-table-column prop="name" label="Room Name"></el-table-column>
        <el-table-column prop="status" label="Status" width="120">
          <template #default="scope">
            <el-tag :type="scope.row.status === 'active' ? 'success' : 'info'">
              {{ scope.row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="participants" label="Participants" width="150"></el-table-column>
        <el-table-column label="Actions" width="200">
          <template #default="scope">
            <el-button 
              size="small" 
              @click="joinRoom(scope.row.id)"
              v-if="scope.row.status === 'active'"
            >
              Join
            </el-button>
            <el-button 
              size="small" 
              @click="manageRoom(scope.row.id)"
              type="primary"
            >
              Manage
            </el-button>
            <el-button 
              size="small" 
              @click="deleteRoom(scope.row.id)"
              type="danger"
              v-if="scope.row.status === 'inactive'"
            >
              Delete
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useStore } from 'vuex'

export default {
  setup() {
    const router = useRouter()
    const store = useStore()
    
    const rooms = ref([])
    const stats = ref({})

    const createRoom = () => {
      router.push('/rooms/create')
    }

    const joinRoom = (roomId) => {
      router.push(`/room/${roomId}`)
    }

    const manageRoom = (roomId) => {
      router.push(`/room/${roomId}/manage`)
    }

    const deleteRoom = (roomId) => {
      // TODO: Implement room deletion
      console.log('Delete room:', roomId)
    }

    const fetchDashboardData = async () => {
      try {
        const response = await store.dispatch('getDashboardData')
        rooms.value = response.rooms || []
        stats.value = response.stats || {}
      } catch (error) {
        console.error('Failed to fetch dashboard data:', error)
      }
    }

    onMounted(() => {
      fetchDashboardData()
    })

    return {
      rooms,
      stats,
      createRoom,
      joinRoom,
      manageRoom,
      deleteRoom
    }
  }
}
</script>

<style scoped>
.dashboard {
  padding: 20px;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.dashboard-header h1 {
  color: #333;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.stat-card {
  text-align: center;
}

.stat-card h3 {
  margin: 0 0 10px 0;
  color: #666;
  font-size: 14px;
  font-weight: normal;
}

.stat-number {
  font-size: 2rem;
  font-weight: bold;
  color: #333;
}

.rooms-section h2 {
  margin-bottom: 20px;
  color: #333;
}
</style>