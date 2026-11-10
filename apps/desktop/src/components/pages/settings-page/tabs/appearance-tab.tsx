import { Check, Monitor, Moon, Sun, type LucideIcon } from 'lucide-react'

import { PageHeading } from '@/components/general/page-heading'
import { useSettings, useUpdateSettings } from '@/hooks/use-settings'
import { cn } from '@/lib/utils'
import type { Theme } from '@/types'

const THEMES: { value: Theme; label: string; icon: LucideIcon; preview: string }[] = [
  {
    value: 'dark',
    label: 'Dark',
    icon: Moon,
    preview: 'bg-[#0d1117] border-[#3d444d] text-[#9198a1]',
  },
  {
    value: 'light',
    label: 'Light',
    icon: Sun,
    preview: 'bg-white border-[#d1d9e0] text-[#1f2328]',
  },
  {
    value: 'system',
    label: 'System',
    icon: Monitor,
    preview: 'bg-[#0d1117] border-[#3d444d] text-[#9198a1]',
  },
]

export function AppearanceTab() {
  const { data: settings } = useSettings()
  const update = useUpdateSettings()
  if (!settings) return null
  return (
    <>
      <PageHeading title="Appearance" description="Choose how GitVault looks." />
      <div className="grid max-w-3xl grid-cols-3 gap-4" role="radiogroup" aria-label="Theme">
        {THEMES.map(({ value, label, icon: Icon, preview }) => {
          const active = settings.theme === value
          return (
            <button
              key={value}
              type="button"
              role="radio"
              aria-checked={active}
              onClick={() => update.mutate({ ...settings, theme: value })}
              className={cn(
                'flex flex-col items-center gap-3 rounded-lg border p-4',
                active ? 'border-info bg-info/10' : 'border-border bg-card hover:bg-muted/40',
              )}
            >
              <span
                className={cn(
                  'flex h-[72px] w-full items-center justify-center rounded-md border',
                  preview,
                )}
              >
                <Icon className="size-6" />
              </span>
              <span className="flex items-center gap-2 text-sm font-medium">
                {label}
                {active ? <Check className="size-3.5 text-success" /> : null}
              </span>
            </button>
          )
        })}
      </div>
    </>
  )
}
