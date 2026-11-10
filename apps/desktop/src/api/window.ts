import { isTauri } from './client'

export type WindowAction = 'minimize' | 'toggleMaximize' | 'close'

/** Minimize / maximize / close the app window (no-op in the browser build). */
export async function windowAction(action: WindowAction): Promise<void> {
  if (!isTauri()) return
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  const current = getCurrentWindow()
  if (action === 'minimize') await current.minimize()
  else if (action === 'toggleMaximize') await current.toggleMaximize()
  else await current.close()
}
