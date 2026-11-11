import { useRouterState } from '@tanstack/react-router'
import { useEffect } from 'react'

import { useSidebar } from '@/stores/sidebar'

/** Collapses the sidebar while a Settings route is open and puts it back afterwards. */
export function useCollapseForSettings() {
  const onSettings = useRouterState({ select: (s) => /\/settings(\/|$)/.test(s.location.pathname) })
  const enter = useSidebar((s) => s.enterSettings)
  const leave = useSidebar((s) => s.leaveSettings)
  useEffect(() => {
    if (onSettings) enter()
    else leave()
  }, [onSettings, enter, leave])
}

/** Ctrl/Cmd+B toggles the sidebar, as in most native apps. */
export function useSidebarShortcut() {
  const toggle = useSidebar((s) => s.toggle)
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'b') {
        event.preventDefault()
        toggle()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [toggle])
}
