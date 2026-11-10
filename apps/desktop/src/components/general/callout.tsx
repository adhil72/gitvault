import { cva, type VariantProps } from 'class-variance-authority'
import type { ReactNode } from 'react'

import { cn } from '@/lib/utils'

const calloutVariants = cva('flex items-start gap-3 rounded-md border p-3 text-sm', {
  variants: {
    tone: {
      info: 'border-info/50 bg-info/10',
      warning: 'border-warning/50 bg-warning/10',
      success: 'border-success/50 bg-success/10',
    },
  },
  defaultVariants: { tone: 'info' },
})

interface CalloutProps extends VariantProps<typeof calloutVariants> {
  icon?: ReactNode
  className?: string
  children: ReactNode
}

/** A tinted message box with an optional leading icon. */
export function Callout({ tone, icon, className, children }: CalloutProps) {
  return (
    <div role="status" className={cn(calloutVariants({ tone }), className)}>
      {icon ? <span className="mt-0.5 shrink-0">{icon}</span> : null}
      <div className="flex-1">{children}</div>
    </div>
  )
}
