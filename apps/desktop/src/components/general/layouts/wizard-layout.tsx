import { Outlet, useRouterState } from '@tanstack/react-router'

import { BrandPanel } from '@/components/general/brand-panel/brand-panel'
import { getWizardPanel } from '@/config/wizards'

/** Brand panel on the left, the current wizard step centered on the right. */
export function WizardLayout() {
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const { headline, description } = getWizardPanel(pathname)
  return (
    <div className="grid min-h-[calc(100vh-var(--titlebar-h))] grid-cols-[560px_1fr]">
      <BrandPanel headline={headline} description={description} />
      <main className="flex items-center justify-center bg-background p-8">
        <Outlet />
      </main>
    </div>
  )
}
