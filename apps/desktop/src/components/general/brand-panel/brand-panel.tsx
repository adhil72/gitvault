import { Logo } from '@/components/general/logo-mark'
import { useAppVersion } from '@/hooks/use-updater'

import { BRAND_FEATURES } from './brand-features'

interface BrandPanelProps {
  headline: string
  description: string
}

/** Left-hand panel of the auth and wizard layouts: logo, pitch and feature list. */
export function BrandPanel({ headline, description }: BrandPanelProps) {
  const { data: version } = useAppVersion()
  return (
    <aside className="flex flex-col justify-between border-r border-sidebar-border bg-sidebar p-14">
      <Logo />
      <div className="flex flex-col gap-7">
        <h2 className="text-[40px] leading-[1.15] font-bold">{headline}</h2>
        <p className="text-base text-muted-foreground">{description}</p>
        <ul className="flex flex-col gap-4">
          {BRAND_FEATURES.map(({ id, title, description: text, icon: Icon }) => (
            <li key={id} className="flex gap-3.5">
              <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-muted">
                <Icon className="size-[18px] text-success" />
              </span>
              <div className="flex flex-col gap-0.5">
                <span className="text-[15px] font-semibold">{title}</span>
                <span className="text-sm text-muted-foreground">{text}</span>
              </div>
            </li>
          ))}
        </ul>
      </div>
      <p className="text-[13px] text-muted-foreground/70">
        {version ? `Version ${version} · ` : ''}Secrets are stored locally, GitHub is only used for
        identity
      </p>
    </aside>
  )
}
