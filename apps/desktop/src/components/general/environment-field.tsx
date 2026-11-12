import { Check, ChevronsUpDown, Plus } from 'lucide-react'
import { useState } from 'react'

import { envDotClass } from '@/components/general/env-badge'
import { NewEnvironmentDialog } from '@/components/general/new-environment-dialog'
import { Button } from '@/components/ui/button'
import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useEnvironmentOptions } from '@/hooks/use-environment-options'
import { cn } from '@/lib/utils'

interface EnvironmentFieldProps {
  id: string
  value: string
  onChange: (value: string) => void
  /** The vault whose existing environments are offered. */
  vaultId: string
}

/** Searchable environment picker (shadcn Popover + Command) that can also create one. */
export function EnvironmentField({ id, value, onChange, vaultId }: EnvironmentFieldProps) {
  const options = useEnvironmentOptions(vaultId)
  const [open, setOpen] = useState(false)
  const [creating, setCreating] = useState(false)
  const [search, setSearch] = useState('')
  // A new environment only exists in the vault once a secret uses it, so keep it
  // listed here until then.
  const [created, setCreated] = useState<string[]>([])
  const all = [...new Set([...options, ...created, ...(value ? [value] : [])])]

  const choose = (next: string) => {
    onChange(next)
    setOpen(false)
  }

  const startCreating = () => {
    setOpen(false)
    setCreating(true)
  }

  const handleCreate = (name: string) => {
    setCreated((prev) => [...prev, name])
    onChange(name)
  }

  return (
    <>
      <Popover
        open={open}
        onOpenChange={(next) => {
          setOpen(next)
          if (!next) setSearch('')
        }}
      >
        <PopoverTrigger asChild>
          <Button
            id={id}
            type="button"
            variant="outline"
            role="combobox"
            aria-expanded={open}
            className="h-10 w-full justify-between bg-inset px-3 font-normal"
          >
            <span
              className={cn('flex items-center gap-2 truncate', !value && 'text-muted-foreground')}
            >
              {value && <span className={cn('size-2 rounded-full', envDotClass(value))} />}
              {value || 'No environment'}
            </span>
            <ChevronsUpDown className="size-4 shrink-0 opacity-50" />
          </Button>
        </PopoverTrigger>
        <PopoverContent align="start" className="w-(--radix-popover-trigger-width) gap-0 p-0">
          <Command>
            <CommandInput
              placeholder="Search environments…"
              value={search}
              onValueChange={setSearch}
            />
            <CommandList>
              <CommandGroup>
                <EnvironmentItem label="None" selected={!value} onSelect={() => choose('')} />
                {all.map((name) => (
                  <EnvironmentItem
                    key={name}
                    label={name}
                    dot={envDotClass(name)}
                    selected={name === value}
                    onSelect={() => choose(name)}
                  />
                ))}
              </CommandGroup>
              <CommandSeparator />
              <CommandGroup forceMount>
                <CommandItem forceMount value="__create__" onSelect={startCreating}>
                  <Plus className="size-4" />
                  {search.trim() ? `Create “${search.trim()}”` : 'Create new environment…'}
                </CommandItem>
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
      <NewEnvironmentDialog
        open={creating}
        onOpenChange={(next) => {
          setCreating(next)
          if (!next) setSearch('')
        }}
        existing={all}
        initialName={search.trim()}
        onCreate={handleCreate}
      />
    </>
  )
}

interface EnvironmentItemProps {
  label: string
  selected: boolean
  dot?: string
  onSelect: () => void
}

function EnvironmentItem({ label, selected, dot, onSelect }: EnvironmentItemProps) {
  return (
    <CommandItem value={label} onSelect={onSelect}>
      {dot && <span className={cn('size-2 rounded-full', dot)} />}
      {label}
      <Check className={cn('ml-auto size-4', selected ? 'opacity-100' : 'opacity-0')} />
    </CommandItem>
  )
}
