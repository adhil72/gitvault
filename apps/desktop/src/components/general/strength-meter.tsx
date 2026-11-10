import { passwordStrength } from '@/lib/password-strength'
import { cn } from '@/lib/utils'

const BAR_TONES = ['bg-destructive', 'bg-destructive', 'bg-warning', 'bg-success', 'bg-success']
const LABEL_TONES = [
  'text-destructive',
  'text-destructive',
  'text-warning',
  'text-success',
  'text-success',
]

/** Four bars and a label that show how strong a password looks. */
export function StrengthMeter({ password }: { password: string }) {
  if (!password) return null
  const { level, label } = passwordStrength(password)
  return (
    <div className="flex flex-col gap-1.5" aria-live="polite">
      <div className="flex gap-1">
        {[1, 2, 3, 4].map((bar) => (
          <span
            key={bar}
            className={cn('h-1.5 w-14 rounded-full bg-muted', bar <= level && BAR_TONES[level])}
          />
        ))}
      </div>
      <span className={cn('text-xs font-medium', LABEL_TONES[level])}>{label}</span>
    </div>
  )
}
