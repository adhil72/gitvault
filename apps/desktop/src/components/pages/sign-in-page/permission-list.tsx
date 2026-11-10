import { Check, Lock } from 'lucide-react'

import { GITHUB_PERMISSIONS } from '@/config/github-permissions'

/** What GitVault asks GitHub for, shown before the user signs in. */
export function PermissionList() {
  return (
    <section className="flex flex-col gap-3 rounded-lg border border-border bg-card p-4">
      <h2 className="text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">
        GitVault will request
      </h2>
      <ul className="flex flex-col gap-3 text-sm">
        {GITHUB_PERMISSIONS.map(({ label, scope }) => (
          <li key={scope} className="flex items-center gap-2.5">
            <Check className="size-4 text-success" />
            <span className="flex-1">{label}</span>
            <code className="font-mono text-xs text-muted-foreground">{scope}</code>
          </li>
        ))}
        <li className="flex items-center gap-2.5">
          <Lock className="size-4 text-muted-foreground" />
          <span>Secrets are encrypted on your device before anything is pushed</span>
        </li>
      </ul>
    </section>
  )
}
