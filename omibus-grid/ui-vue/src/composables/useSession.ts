import { ref, readonly } from 'vue'

export interface SessionUser {
  id: number
  displayName: string
}

const _user = ref<SessionUser | null>(null)

export function useSession() {
  function setUser(u: SessionUser) { _user.value = u }
  function clearUser() { _user.value = null }
  return { user: readonly(_user), setUser, clearUser }
}
