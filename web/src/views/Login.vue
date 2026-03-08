<template>
  <div class="login">
    <div class="login-container">
      <h1>DRMP Login</h1>
      <el-form :model="loginForm" :rules="loginRules" ref="loginFormRef" class="login-form">
        <el-form-item prop="email">
          <el-input v-model="loginForm.email" placeholder="Email" prefix-icon="el-icon-user"></el-input>
        </el-form-item>
        <el-form-item prop="password">
          <el-input 
            v-model="loginForm.password" 
            type="password" 
            placeholder="Password" 
            prefix-icon="el-icon-lock"
            @keyup.enter="handleLogin"
          ></el-input>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleLogin" class="login-btn" :loading="loading">
            Login
          </el-button>
        </el-form-item>
        <el-form-item>
          <el-button type="text" @click="goToRegister" class="register-link">
            Don't have an account? Register
          </el-button>
        </el-form-item>
      </el-form>
    </div>
  </div>
</template>

<script>
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { useStore } from 'vuex'

export default {
  setup() {
    const router = useRouter()
    const store = useStore()
    
    const loginFormRef = ref(null)
    const loading = ref(false)

    const loginForm = reactive({
      email: '',
      password: ''
    })

    const loginRules = {
      email: [
        { required: true, message: 'Please enter email', trigger: 'blur' },
        { type: 'email', message: 'Please enter valid email', trigger: 'blur' }
      ],
      password: [
        { required: true, message: 'Please enter password', trigger: 'blur' },
        { min: 6, message: 'Password must be at least 6 characters', trigger: 'blur' }
      ]
    }

    const handleLogin = () => {
      loginFormRef.value.validate((valid) => {
        if (valid) {
          loading.value = true
          
          store.dispatch('login', {
            email: loginForm.email,
            password: loginForm.password
          })
          .then(() => {
            loading.value = false
            router.push('/dashboard')
          })
          .catch((error) => {
            loading.value = false
            console.error('Login failed:', error)
          })
        }
      })
    }

    const goToRegister = () => {
      router.push('/register')
    }

    return {
      loginFormRef,
      loginForm,
      loginRules,
      loading,
      handleLogin,
      goToRegister
    }
  }
}
</script>

<style scoped>
.login {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.login-container {
  background: white;
  padding: 40px;
  border-radius: 10px;
  box-shadow: 0 10px 30px rgba(0,0,0,0.2);
  width: 100%;
  max-width: 400px;
}

h1 {
  text-align: center;
  margin-bottom: 30px;
  color: #333;
}

.login-form {
  margin-top: 20px;
}

.login-btn {
  width: 100%;
  font-weight: bold;
}

.register-link {
  display: block;
  text-align: center;
  color: #667eea;
}

.register-link:hover {
  color: #764ba2;
}
</style>