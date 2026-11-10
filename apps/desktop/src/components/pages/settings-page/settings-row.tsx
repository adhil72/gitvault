import type { ReactNode } from 'react'

interface SettingsRowProps {
  title: string
  description?: string
  children: ReactNode
}

/** A titled setting with its control on the right. Rows stack inside a bordered card. */
export function SettingsRow({ title, description, children }: SettingsRowProps) {
  return (
    <div className="flex items-center gap-4 border-b border-border px-5 py-4 last:border-b-0">
      <div className="flex flex-1 flex-col gap-0.5">
        <span className="text-[15px] font-semibold">{title}</span>
        {description ? (
          <span className="text-[13px] text-muted-foreground">{description}</span>
        ) : null}
      </div>
      {children}
    </div>
  )
}

export function SettingsCard({ children }: { children: ReactNode }) {
  return (
    <section className="flex flex-col rounded-lg border border-border bg-card">{children}</section>
  )
}
