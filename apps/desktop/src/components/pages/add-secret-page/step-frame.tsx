import type { FormEvent, ReactNode } from 'react'

interface StepFrameProps {
  title: string
  description: string
  /** Rendered under the fields, normally a `StepActions`. */
  actions: ReactNode
  onSubmit?: (event: FormEvent) => void
  children: ReactNode
}

/** Shared layout of every wizard step: heading, fields, then actions, all on the page itself. */
export function StepFrame({ title, description, actions, onSubmit, children }: StepFrameProps) {
  return (
    <form
      onSubmit={onSubmit ?? ((event) => event.preventDefault())}
      className="flex flex-col gap-9"
    >
      <header className="flex flex-col gap-2.5">
        <h1 className="text-[34px] leading-[1.1] font-semibold tracking-[-0.03em] text-balance">
          {title}
        </h1>
        <p className="text-base text-pretty text-muted-foreground">{description}</p>
      </header>
      <div className="flex flex-col gap-6">{children}</div>
      <div className="flex items-center justify-between gap-3">{actions}</div>
    </form>
  )
}
