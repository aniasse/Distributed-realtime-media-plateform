<template>
  <div class="rooms">
    <div class="rooms-header">
      <h1>Rooms</h1>
      <el-button type="primary" @click="createRoom">Create Room</el-button>
    </div>
    
    <div class="rooms-grid">
      <el-card v-for="room in rooms" :key="room.id" class="room-card">
        <div class="room-header">
          <h3>{{ room.name }}</h3>
          <el-tag :type="room.status === 'active' ? 'success' : 'info'">
            {{ room.status }}
          </el-tag>
        </div>
        <p class="room-description">{{ room.description }}</p>
        <div class="room-meta">
          <span class="participant-count">
            <i class="el-icon-user"></i> {{ room.participants }}/{{ room.maxParticipants }}
          </span>
          <span class="room-id">ID: {{ room.id }}</span>
        </div>
        <div class="room-actions">
          <el-button 
            type="primary" 
            @click="joinRoom(room.id)"
            v-if="room.status === 'active'"
            size="small"
          >
            Join
          </el-button>
          <el-button 
            type="warning" 
            @click="manageRoom(room.id)"
            size="small"
          >
            Manage
          </el-button>
        </div>
      </el-card>
    </div>
    
    <el-pagination
      v-if="totalPages > 1"
      @current-change="handlePageChange"
      :current-page="currentPage"
      :page-size="pageSize"
      :total="totalItems"
      layout="prev, pager, next"
    ></el-pagination>
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
    const currentPage = ref(1)
    const pageSize = ref(12)
    const totalItems = ref(0)
    const totalPages = ref(0)

    const fetchRooms = async (page = 1) => {
      try {
        const response = await store.dispatch('getRooms', {
          page,
          pageSize: pageSize.value
        })
        
        rooms.value = response.rooms || []
        totalItems.value = response.totalItems || 0
        totalPages.value = Math.ceil(totalItems.value / pageSize.value)
        currentPage.value = page
      } catch (error) {
        console.error('Failed to fetch rooms:', error)
      }
    }

    const createRoom = () => {
      router.push('/rooms/create')
    }

    const joinRoom = (roomId) => {
      router.push(`/room/${roomId}`)
    }

    const manageRoom = (roomId) => {
      router.push(`/room/${roomId}/manage`)
    }

    const handlePageChange = (page) => {
      fetchRooms(page)
    }

    onMounted(() => {
      fetchRooms()
    })

    return {
      rooms,
      currentPage,
      pageSize,
      totalItems,
      totalPages,
      createRoom,
      joinRoom,
      manageRoom,
      handlePageChange
    }
  }
}
</script>

<style scoped>
.rooms {
  padding: 20px;
}

.rooms-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.rooms-header h1 {
  color: #333;
}

.rooms-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.room-card {
  padding: 20px;
}

.room-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.room-header h3 {
  margin: 0;
  color: #333;
}

.room-description {
  color: #666;
  margin: 10px 0;
  font-size: 14px;
}

.room-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin: 15px 0;
  font-size: 12px;
  color: #999;
}

.participant-count {
  color: #409eff;
}

.room-actions {
  display: flex;
  gap: 10px;
}

.el-pagination {
  display: flex;
  justify-content: center;
}
</style>