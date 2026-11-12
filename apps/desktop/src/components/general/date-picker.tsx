import { CalendarDays, ChevronDown, X } from 'lucide-react'
import { useState } from 'react'

import { Button } from '@/components/ui/button'
import { Calendar } from '@/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { displayIso, todayIso } from '@/lib/calendar'
import { cn } from '@/lib/utils'

interface DatePickerProps {
  id: string
  /** "YYYY-MM-DD" or "" when empty. */
  value: string
  onChange: (value: string) => void
  placeholder?: string
  /** Extra classes for the trigger, to restyle it per page. */
  className?: string
}

/** The date field used everywhere: a trigger showing "Dec 04, 2026" that opens a calendar popover. */
export function DatePicker({ id, value, onChange, placeholder, className }: DatePickerProps) {
  const [open, setOpen] = useState(false)
  const pick = (iso: string) => {
    onChange(iso)
    setOpen(false)
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          id={id}
          type="button"
          variant="outline"
          className={cn('h-10 w-full justify-between bg-inset px-3 font-normal', className)}
        >
          <span className={cn('flex items-center gap-2', !value && 'text-muted-foreground')}>
            <CalendarDays className="size-4" />
            {value ? displayIso(value) : (placeholder ?? 'Pick a date')}
          </span>
          <ChevronDown className="size-4 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-auto gap-3">
        <Calendar value={value} onSelect={pick} />
        <div className="flex justify-between">
          <Button type="button" variant="ghost" size="sm" onClick={() => pick(todayIso())}>
            Today
          </Button>
          {value ? (
            <Button type="button" variant="ghost" size="sm" onClick={() => pick('')}>
              <X /> Clear
            </Button>
          ) : null}
        </div>
      </PopoverContent>
    </Popover>
  )
}
