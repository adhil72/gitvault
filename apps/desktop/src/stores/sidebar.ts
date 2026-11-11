import { create } from 'zustand'

/**
 * Whether the vault sidebar is collapsed to its icon rail. Opening Settings collapses it to give
 * the settings pane room; leaving restores what the user had, unless they toggled it by hand.
 */
interface SidebarState {
  collapsed: boolean
  /** The user's choice from before Settings took over, or null when nothing is pending. */
  restoreTo: boolean | null
  toggle: () => void
  enterSettings: () => void
  leaveSettings: () => void
}

export const useSidebar = create<SidebarState>((set) => ({
  collapsed: false,
  restoreTo: null,
  // A manual toggle is an explicit choice, so it cancels the automatic restore.
  toggle: () => set((s) => ({ collapsed: !s.collapsed, restoreTo: null })),
  enterSettings: () =>
    set((s) => (s.restoreTo === null ? { restoreTo: s.collapsed, collapsed: true } : s)),
  leaveSettings: () =>
    set((s) => (s.restoreTo === null ? s : { collapsed: s.restoreTo, restoreTo: null })),
}))
