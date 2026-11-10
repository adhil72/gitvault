import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useState } from 'react'

import { Button } from '@/components/ui/button'
import { monthGrid, monthLabel, parseIso, shiftMonth, todayIso } from '@/lib/calendar'
import { cn } from '@/lib/utils'

const WEEKDAYS = ['Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa', 'Su']

interface CalendarProps {
  /** Selected day as "YYYY-MM-DD", or "". */
  value: string
  onSelect: (iso: string) => void
}

/** A month grid (Monday first) with previous/next navigation. Written for GitVault, no date library. */
export function Calendar({ value, onSelect }: CalendarProps) {
  const initial = parseIso(value) ?? parseIso(todayIso())
  const [view, setView] = useState({ year: initial?.year ?? 2026, month: initial?.month ?? 0 })
  const today = todayIso()
  const move = (by: number) => setView((v) => shiftMonth(v.year, v.month, by))

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label="Previous month"
          onClick={() => move(-1)}
        >
          <ChevronLeft />
        </Button>
        <span className="text-sm font-semibold">{monthLabel(view.year, view.month)}</span>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label="Next month"
          onClick={() => move(1)}
        >
          <ChevronRight />
        </Button>
      </div>
      <div className="grid grid-cols-7 gap-1 text-center text-xs text-muted-foreground">
        {WEEKDAYS.map((day) => (
          <span key={day}>{day}</span>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-1">
        {monthGrid(view.year, view.month).map((cell) => (
          <button
            key={cell.iso}
            type="button"
            onClick={() => onSelect(cell.iso)}
            aria-pressed={cell.iso === value}
            className={cn(
              'flex size-9 items-center justify-center rounded-md text-sm transition-colors hover:bg-muted',
              !cell.inMonth && 'text-muted-foreground/50',
              cell.iso === today && 'border border-info',
              cell.iso === value && 'bg-primary text-primary-foreground hover:bg-primary',
            )}
          >
            {cell.day}
          </button>
        ))}
      </div>
    </div>
  )
}
