import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

const TONES: Record<string, string> = {
  production: 'border-destructive/60 bg-destructive/10 text-destructive',
  prod: 'border-destructive/60 bg-destructive/10 text-destructive',
  staging: 'border-warning/60 bg-warning/10 text-warning',
  development: 'border-info/60 bg-info/10 text-info',
  dev: 'border-info/60 bg-info/10 text-info',
  shared: 'border-done/60 bg-done/10 text-done',
}

const NEUTRAL = 'border-border bg-muted text-muted-foreground'

/** Color-coded pill for an environment (or any short label). */
export function EnvBadge({ name, className }: { name: string; className?: string }) {
  const tone = TONES[name.trim().toLowerCase()] ?? NEUTRAL
  return (
    <Badge
      variant="outline"
      className={cn('rounded-full px-2.5 text-xs font-medium', tone, className)}
    >
      {name}
    </Badge>
  )
}

/** The dot color for an environment in lists. */
export function envDotClass(name: string): string {
  const key = name.trim().toLowerCase()
  if (key === 'production' || key === 'prod') return 'bg-destructive'
  if (key === 'staging') return 'bg-warning'
  if (key === 'development' || key === 'dev') return 'bg-info'
  return 'bg-muted-foreground'
}
