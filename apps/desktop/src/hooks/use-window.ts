import { isTauri } from '@/api/client'
import { windowAction, type WindowAction } from '@/api/window'

/** Window controls for the custom title bar; `available` is false in the browser build. */
export function useWindowControls(): { available: boolean; run: (action: WindowAction) => void } {
  return { available: isTauri(), run: (action) => void windowAction(action) }
}

export type { WindowAction }
