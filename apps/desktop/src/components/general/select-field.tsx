import { Check, ChevronsUpDown } from 'lucide-react'
import { useState } from 'react'

import { Button } from '@/components/ui/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { cn } from '@/lib/utils'

export interface SelectOption {
  value: string
  label: string
}

interface SelectFieldProps {
  id: string
  value: string
  onChange: (value: string) => void
  options: readonly SelectOption[]
  placeholder?: string
  /** Adds a "None" choice that maps to the empty string. */
  allowEmpty?: boolean
  /** Extra classes for the trigger, to restyle it per page. */
  className?: string
}

/** A searchable single-choice combobox (shadcn Popover + Command). */
export function SelectField({
  id,
  value,
  onChange,
  options,
  placeholder,
  allowEmpty,
  className,
}: SelectFieldProps) {
  const [open, setOpen] = useState(false)
  const choices = allowEmpty ? [{ value: '', label: 'None' }, ...options] : options
  const selected = options.find((option) => option.value === value)

  const choose = (next: string) => {
    onChange(next)
    setOpen(false)
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          id={id}
          type="button"
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn('h-10 w-full justify-between bg-inset px-3 font-normal', className)}
        >
          <span className={cn('truncate', !selected && 'text-muted-foreground')}>
            {selected?.label ?? placeholder ?? 'Select…'}
          </span>
          <ChevronsUpDown className="size-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-(--radix-popover-trigger-width) gap-0 p-0">
        <Command>
          <CommandInput placeholder="Search…" />
          <CommandList>
            <CommandEmpty>No results.</CommandEmpty>
            <CommandGroup>
              {choices.map((option) => (
                <CommandItem
                  key={option.value || '__none__'}
                  value={option.label}
                  onSelect={() => choose(option.value)}
                >
                  {option.label}
                  <Check
                    className={cn(
                      'ml-auto size-4',
                      option.value === value ? 'opacity-100' : 'opacity-0',
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
