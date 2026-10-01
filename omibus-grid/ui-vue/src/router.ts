import { createRouter, createWebHashHistory } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useSession } from './composables/useSession'
import GridPage      from './pages/GridPage.vue'
import ChartsPage    from './pages/ChartsPage.vue'
import AiPage        from './pages/AiPage.vue'
import AuthPage      from './pages/AuthPage.vue'
import ProfilePage   from './pages/ProfilePage.vue'
import OrderbookPage from './pages/OrderbookPage.vue'
import TradePage        from './pages/TradePage.vue'
import GridBotPlusPage  from './pages/GridBotPlusPage.vue'
import GridBotPage         from './pages/GridBotPage.vue'
import GridBotFormPlusPage from './pages/GridBotFormPlusPage.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/',          redirect: '/grid' },
    { path: '/auth',      component: AuthPage,      meta: { public: true } },
    { path: '/grid',      component: GridPage },
    { path: '/charts',    component: ChartsPage },
    { path: '/orderbook', component: OrderbookPage },
    { path: '/trade',       component: TradePage },
    { path: '/gridbotplus', component: GridBotPlusPage },
    { path: '/gridbot',        component: GridBotPage         },
    { path: '/gridbotformplus', component: GridBotFormPlusPage },
    { path: '/ai',        component: AiPage },
    { path: '/profile',   component: ProfilePage },
  ],
})

// Guard: skip entirely if auth module is disabled in modules.json
let authModuleEnabled: boolean | null = null
async function isAuthEnabled(): Promise<boolean> {
  if (authModuleEnabled !== null) return authModuleEnabled
  try {
    const cfg = await invoke<{ auth: boolean }>('get_modules')
    authModuleEnabled = cfg.auth ?? true
  } catch {
    authModuleEnabled = true
  }
  return authModuleEnabled
}

router.beforeEach(async (to) => {
  if (to.meta.public) return true
  if (!(await isAuthEnabled())) return true   // auth disabled → free access

  const { user, setUser } = useSession()
  if (user.value) return true

  // Restore session after hot-reload
  try {
    const whoami = await invoke<{ user_id: number; display_name: string } | null>('auth_whoami')
    if (whoami) {
      setUser({ id: whoami.user_id, displayName: whoami.display_name })
      return true
    }
  } catch { /* ignore */ }

  return '/auth'
})

export default router
