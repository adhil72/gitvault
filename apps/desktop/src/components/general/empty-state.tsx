import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'

interface EmptyStateProps {
  icon: LucideIcon
  title: string
  description: string
  children?: ReactNode
  /** Drops the framed tile around the icon, leaving just the glyph. */
  plainIcon?: boolean
}

const RISE = 'animate-rise motion-reduce:animate-none'

/** Centered message shown when a list has nothing in it yet. */
export function EmptyState({
  icon: Icon,
  title,
  description,
  children,
  plainIcon = false,
}: EmptyStateProps) {
  const icon = <Icon className="size-7 text-success" strokeWidth={1.75} />
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-2 p-10 pb-24 text-center">
      {plainIcon ? (
        <div className={RISE}>{icon}</div>
      ) : (
        // The soft glow lifts the tile off the canvas without adding a border.
        <span
          className={`${RISE} mb-4 flex size-16 items-center justify-center rounded-[20px] bg-gradient-to-b from-success/15 to-success/5 shadow-[0_8px_32px_-8px] shadow-success/30 ring-1 ring-success/20 ring-inset`}
        >
          {icon}
        </span>
      )}
      <h2 className={`${RISE} text-[22px] font-semibold tracking-[-0.02em] [animation-delay:60ms]`}>
        {title}
      </h2>
      <p
        className={`${RISE} max-w-sm text-[15px] leading-relaxed text-pretty text-muted-foreground [animation-delay:110ms]`}
      >
        {description}
      </p>
      {children ? (
        <div className={`${RISE} mt-4 flex items-center gap-2 [animation-delay:170ms]`}>
          {children}
        </div>
      ) : null}
    </div>
  )
}
