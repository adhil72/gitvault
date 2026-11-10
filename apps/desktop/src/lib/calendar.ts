/** Date helpers for the date picker. Dates travel as "YYYY-MM-DD" strings (always UTC-free, no time zones). */

export interface CalendarDay {
  iso: string
  day: number
  inMonth: boolean
}

const pad = (n: number) => String(n).padStart(2, '0')

export const toIso = (year: number, month: number, day: number) =>
  `${year}-${pad(month + 1)}-${pad(day)}`

export function parseIso(value: string): { year: number; month: number; day: number } | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value)
  if (!match) return null
  return { year: Number(match[1]), month: Number(match[2]) - 1, day: Number(match[3]) }
}

export const todayIso = () => {
  const now = new Date()
  return toIso(now.getFullYear(), now.getMonth(), now.getDate())
}

/** Six weeks (Monday first) covering the month, so the grid never changes height. */
export function monthGrid(year: number, month: number): CalendarDay[] {
  const first = new Date(Date.UTC(year, month, 1))
  const offset = (first.getUTCDay() + 6) % 7
  return Array.from({ length: 42 }, (_, i) => {
    const date = new Date(Date.UTC(year, month, 1 - offset + i))
    return {
      iso: toIso(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()),
      day: date.getUTCDate(),
      inMonth: date.getUTCMonth() === month,
    }
  })
}

export const shiftMonth = (year: number, month: number, by: number) => {
  const date = new Date(Date.UTC(year, month + by, 1))
  return { year: date.getUTCFullYear(), month: date.getUTCMonth() }
}

/** "Dec 04, 2026" for a "YYYY-MM-DD" value. */
export function displayIso(value: string): string {
  const parsed = parseIso(value)
  if (!parsed) return ''
  return new Date(Date.UTC(parsed.year, parsed.month, parsed.day)).toLocaleDateString('en-US', {
    month: 'short',
    day: '2-digit',
    year: 'numeric',
    timeZone: 'UTC',
  })
}

export const monthLabel = (year: number, month: number) =>
  new Date(Date.UTC(year, month, 1)).toLocaleDateString('en-US', {
    month: 'long',
    year: 'numeric',
    timeZone: 'UTC',
  })
