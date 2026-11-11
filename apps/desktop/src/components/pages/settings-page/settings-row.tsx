import type { ReactNode } from 'react'

interface SettingsRowProps {
  title: string
  description?: string
  children: ReactNode
}

/** A titled setting with its control on the right. Rows stack inside a grouped card. */
export function SettingsRow({ title, description, children }: SettingsRowProps) {
  return (
    <div className="flex items-center gap-6 px-5 py-3.5 not-last:border-b not-last:border-border/40">
      <div className="flex flex-1 flex-col gap-0.5">
        <span className="text-sm font-medium">{title}</span>
        {description ? (
          <span className="text-[13px] leading-snug text-muted-foreground">{description}</span>
        ) : null}
      </div>
      {children}
    </div>
  )
}

/** Grouped surface: lifted by tone and a hairline ring rather than a hard border. */
export function SettingsCard({ children }: { children: ReactNode }) {
  return (
    <section className="flex flex-col overflow-hidden rounded-xl bg-card ring-1 ring-border/50">
      {children}
    </section>
  )
}

interface SettingsGroupProps {
  title: string
  /** Quiet explanation under the card, like the footnote of a grouped list. */
  footer?: string
  children: ReactNode
}

/** A labelled card: the label sits above it, the explanation below, so rows stay uncluttered. */
export function SettingsGroup({ title, footer, children }: SettingsGroupProps) {
  return (
    <div className="flex flex-col gap-2">
      <h2 className="px-1 text-[13px] font-medium text-muted-foreground">{title}</h2>
      <SettingsCard>{children}</SettingsCard>
      {footer ? (
        <p className="px-1 text-xs leading-snug text-pretty text-muted-foreground">{footer}</p>
      ) : null}
    </div>
  )
}
