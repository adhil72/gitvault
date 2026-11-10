import { Minus, Square, X } from 'lucide-react'

import { LogoMark } from '@/components/general/logo-mark'
import { useWindowControls, type WindowAction } from '@/hooks/use-window'
import { cn } from '@/lib/utils'

const CONTROLS: { action: WindowAction; label: string; icon: typeof X }[] = [
  { action: 'minimize', label: 'Minimize', icon: Minus },
  { action: 'toggleMaximize', label: 'Maximize', icon: Square },
  { action: 'close', label: 'Close', icon: X },
]

/** Draggable window bar styled like the rest of the app, with its own window controls. */
export function TitleBar() {
  const { available, run } = useWindowControls()
  if (!available) return null
  return (
    <div
      data-tauri-drag-region
      onDoubleClick={() => run('toggleMaximize')}
      className="fixed inset-x-0 top-0 z-[60] flex h-(--titlebar-h) items-center justify-between border-b border-border bg-inset select-none"
    >
      <div data-tauri-drag-region className="pointer-events-none flex items-center gap-2 pl-3">
        <LogoMark className="size-4" />
        <span className="text-xs font-medium text-muted-foreground">GitVault</span>
      </div>
      <div className="flex h-full">
        {CONTROLS.map(({ action, label, icon: Icon }) => (
          <button
            key={action}
            type="button"
            aria-label={label}
            onClick={() => run(action)}
            onDoubleClick={(event) => event.stopPropagation()}
            className={cn(
              'flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-muted hover:text-foreground',
              action === 'close' && 'hover:bg-destructive hover:text-white',
            )}
          >
            <Icon className="size-3.5" />
          </button>
        ))}
      </div>
    </div>
  )
}
