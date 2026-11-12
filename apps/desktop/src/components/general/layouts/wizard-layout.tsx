import { Outlet } from '@tanstack/react-router'

/** Wizard pages are shown centered on their own, with no side panel. */
export function WizardLayout() {
  return (
    <main className="flex min-h-[calc(100vh-var(--titlebar-h))] items-center justify-center bg-background p-8">
      <Outlet />
    </main>
  )
}
