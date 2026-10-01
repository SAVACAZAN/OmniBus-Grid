<template>
  <div class="auth-shell">
    <div class="auth-card">
      <div class="auth-logo">▦ Omibus Grid</div>

      <!-- Auto-login in progress -->
      <div v-if="autoLogging" class="auth-auto">
        <span class="auth-spinner">⟳</span> Signing in…
      </div>

      <template v-else>
        <div class="auth-tabs">
          <button :class="['auth-tab', { active: mode === 'login' }]" @click="mode = 'login'">Login</button>
          <button :class="['auth-tab', { active: mode === 'register' }]" @click="mode = 'register'">Register</button>
        </div>

        <form class="auth-form" @submit.prevent="submit">
          <label class="auth-label">
            Username
            <input v-model="form.username" class="auth-input" autocomplete="username"
                   placeholder="Enter username" :disabled="busy" />
          </label>

          <label v-if="mode === 'register'" class="auth-label">
            Display name
            <input v-model="form.displayName" class="auth-input" autocomplete="name"
                   placeholder="Optional display name" :disabled="busy" />
          </label>

          <label class="auth-label">
            Password
            <input v-model="form.password" type="password" class="auth-input" autocomplete="current-password"
                   placeholder="Enter password" :disabled="busy" />
          </label>

          <label v-if="mode === 'login'" class="auth-remember">
            <input type="checkbox" v-model="rememberMe" :disabled="busy" />
            Remember me
          </label>

          <p v-if="error" class="auth-error">{{ error }}</p>

          <button type="submit" class="auth-submit" :disabled="busy">
            {{ busy ? '…' : (mode === 'login' ? 'Login' : 'Create account') }}
          </button>
        </form>

        <p v-if="mode === 'register'" class="auth-hint">
          Password encrypts your API keys — it cannot be recovered if lost.
        </p>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useInvoke } from '../composables/useTauri'
import { useSession } from '../composables/useSession'

const invoke = useInvoke()
const router = useRouter()
const { setUser } = useSession()

const mode       = ref<'login' | 'register'>('login')
const busy       = ref(false)
const autoLogging = ref(false)
const error      = ref('')
const rememberMe = ref(true)

const form = reactive({ username: '', password: '', displayName: '' })

watch(mode, () => { error.value = ''; form.password = '' })

// ── Auto-login on mount ───────────────────────────────────────────────────────
onMounted(async () => {
  autoLogging.value = true
  try {
    const result = await invoke<{ user_id: number; display_name: string } | null>('auth_autologin')
    if (result) {
      setUser({ id: result.user_id, displayName: result.display_name })
      router.replace('/grid')
      return
    }
  } catch { /* no saved creds or stale — show form */ }
  autoLogging.value = false
})

// ── Manual submit ─────────────────────────────────────────────────────────────
async function submit() {
  error.value = ''
  if (!form.username.trim() || !form.password) {
    error.value = 'Username and password are required.'
    return
  }
  busy.value = true
  try {
    let result: { user_id: number; display_name: string }
    if (mode.value === 'login') {
      result = await invoke<{ user_id: number; display_name: string }>('auth_login', {
        username: form.username.trim(),
        password: form.password,
      })
      if (rememberMe.value) {
        await invoke('auth_save_credentials', { username: form.username.trim(), password: form.password })
      } else {
        await invoke('auth_forget_credentials').catch(() => {})
      }
    } else {
      result = await invoke<{ user_id: number; display_name: string }>('auth_register', {
        username: form.username.trim(),
        password: form.password,
        displayName: form.displayName.trim(),
      })
    }
    setUser({ id: result.user_id, displayName: result.display_name || form.username.trim() })
    router.replace('/grid')
  } catch (e: any) {
    error.value = String(e)
  } finally {
    busy.value = false
  }
}
</script>

<style scoped>
.auth-shell {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: var(--bg);
}

.auth-card {
  width: 340px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 32px 28px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.auth-logo {
  font-size: 1.3rem;
  font-weight: 700;
  color: var(--accent);
  text-align: center;
  letter-spacing: .04em;
}

.auth-auto {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: .9rem;
  padding: 24px 0;
}

.auth-spinner {
  display: inline-block;
  animation: spin 1s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

.auth-tabs {
  display: flex;
  gap: 4px;
  background: var(--bg);
  border-radius: 6px;
  padding: 3px;
}

.auth-tab {
  flex: 1;
  padding: 6px 0;
  border: none;
  background: transparent;
  color: var(--text-muted);
  border-radius: 5px;
  cursor: pointer;
  font-size: .85rem;
  transition: background .15s, color .15s;
}

.auth-tab.active {
  background: var(--surface);
  color: var(--text);
  box-shadow: 0 1px 3px rgba(0,0,0,.4);
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.auth-label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: .8rem;
  color: var(--text-muted);
}

.auth-input {
  padding: 8px 10px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text);
  font-size: .9rem;
  outline: none;
  transition: border-color .15s;
}

.auth-input:focus { border-color: var(--accent); }
.auth-input:disabled { opacity: .5; }

.auth-remember {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: .82rem;
  color: var(--text-muted);
  cursor: pointer;
  user-select: none;
}

.auth-remember input[type="checkbox"] {
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
  cursor: pointer;
}

.auth-error {
  color: var(--danger, #f55);
  font-size: .82rem;
  margin: 0;
}

.auth-submit {
  padding: 9px;
  background: var(--accent);
  color: #000;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  font-size: .9rem;
  cursor: pointer;
  transition: opacity .15s;
}

.auth-submit:disabled { opacity: .5; cursor: not-allowed; }

.auth-hint {
  font-size: .75rem;
  color: var(--text-muted);
  text-align: center;
  margin: 0;
}
</style>
