import { invoke as tauriInvoke } from '@tauri-apps/api/core'

export function useInvoke() {
  return <T>(cmd: string, args?: Record<string, unknown>): Promise<T> =>
    tauriInvoke<T>(cmd, args)
}
