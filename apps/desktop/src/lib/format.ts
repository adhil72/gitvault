const MINUTE = 60
const HOUR = 3600
const DAY = 86_400

/** "3 days ago", "just now" ... for a Unix timestamp in seconds. */
export function timeAgo(unixSeconds: number, nowMs: number = Date.now()): string {
  const seconds = Math.max(Math.floor(nowMs / 1000) - unixSeconds, 0)
  const units: [number, string][] = [
    [DAY, 'day'],
    [HOUR, 'hour'],
    [MINUTE, 'minute'],
  ]
  for (const [size, name] of units) {
    if (seconds >= size) {
      const count = Math.floor(seconds / size)
      return `${count} ${name}${count === 1 ? '' : 's'} ago`
    }
  }
  return 'just now'
}

/** "Dec 04, 2026" */
export function formatDate(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleDateString('en-US', {
    month: 'short',
    day: '2-digit',
    year: 'numeric',
  })
}

/** "2026-12-04" for <input type="date">. */
export function toDateInput(unixSeconds: number | null): string {
  return unixSeconds === null ? '' : new Date(unixSeconds * 1000).toISOString().slice(0, 10)
}

/** Unix seconds (UTC midnight) from an <input type="date"> value, or null. */
export function fromDateInput(value: string): number | null {
  if (!value) return null
  const ms = Date.parse(`${value}T00:00:00Z`)
  return Number.isNaN(ms) ? null : Math.floor(ms / 1000)
}

/** Plain-language expiry, e.g. "Expires in 12 days" or "Expired 3 days ago". */
export function describeExpiry(days: number | null): string | null {
  if (days === null) return null
  if (days < 0) return `Expired ${-days} day${days === -1 ? '' : 's'} ago`
  if (days === 0) return 'Expires today'
  return `Expires in ${days} day${days === 1 ? '' : 's'}`
}
