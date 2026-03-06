<template>
  <div id="app">
    <el-container>
      <el-header>
        <el-menu
          :default-active="activeIndex"
          class="el-menu-demo"
          mode="horizontal"
          @select="handleSelect"
        >
          <el-menu-item index="1">DRMP</el-menu-item>
          <el-menu-item index="2">Dashboard</el-menu-item>
          <el-menu-item index="3">Rooms</el-menu-item>
          <el-menu-item index="4">Recording</el-menu-item>
          <el-menu-item index="5">Settings</el-menu-item>
          <el-menu-item index="6" v-if="isLoggedIn" @click="logout">Logout</el-menu-item>
        </el-menu>
      </el-header>
      <el-main>
        <router-view />
      </el-main>
    </el-container>
  </div>
</template>

<script>
import { ref, computed } from 'vue'
import { useStore } from 'vuex'

export default {
  setup() {
    const store = useStore()
    const activeIndex = ref('1')

    const isLoggedIn = computed(() => store.getters.isLoggedIn)

    const handleSelect = (key) => {
      switch (key) {
        case '1':
          break
        case '2':
          router.push('/dashboard')
          break
        case '3':
          router.push('/rooms')
          break
        case '4':
          router.push('/recording')
          break
        case '5':
          router.push('/settings')
          break
      }
    }

    const logout = () => {
      store.dispatch('logout')
      router.push('/login')
    }

    return {
      activeIndex,
      isLoggedIn,
      handleSelect,
      logout
    }
  }
}
</script>