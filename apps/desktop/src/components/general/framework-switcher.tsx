import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { CONNECT_TARGETS } from '@/config/connect-targets'

interface FrameworkSwitcherProps {
  value: string
  onValueChange: (id: string) => void
}

/** Card-style picker for the framework a snippet is shown for. Pair it with its own content. */
export function FrameworkSwitcher({ value, onValueChange }: FrameworkSwitcherProps) {
  return (
    <Tabs value={value} onValueChange={onValueChange}>
      <TabsList
        aria-label="Framework"
        className="grid h-auto w-full grid-cols-2 gap-2 bg-transparent p-0 sm:grid-cols-3"
      >
        {CONNECT_TARGETS.map(({ id, label, tagline, icon: Icon }) => (
          <TabsTrigger
            key={id}
            value={id}
            className="h-auto flex-none justify-start gap-3 rounded-lg border-border px-3 py-2.5 text-left data-active:border-primary data-active:bg-primary/10 dark:data-active:border-primary dark:data-active:bg-primary/10"
          >
            <Icon className="size-5 shrink-0" />
            <span className="flex flex-col">
              <span className="font-medium">{label}</span>
              <span className="text-xs font-normal text-muted-foreground">{tagline}</span>
            </span>
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  )
}
