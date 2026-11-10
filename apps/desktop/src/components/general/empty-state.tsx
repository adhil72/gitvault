import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'

interface EmptyStateProps {
  icon: LucideIcon
  title: string
  description: string
  children?: ReactNode
}

/** Centered message shown when a list has nothing in it yet. */
export function EmptyState({ icon: Icon, title, description, children }: EmptyStateProps) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 p-10 text-center">
      <span className="flex size-[72px] items-center justify-center rounded-2xl border border-border bg-card">
        <Icon className="size-8 text-success" />
      </span>
      <h2 className="text-2xl font-bold">{title}</h2>
      <p className="max-w-md text-muted-foreground">{description}</p>
      {children ? <div className="flex gap-3">{children}</div> : null}
    </div>
  )
}
