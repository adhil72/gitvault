interface LogoMarkProps {
  className?: string
}

/** The GitVault mark: a hexagon (follows the text colour) around a green keyhole. */
export function LogoMark({ className }: LogoMarkProps) {
  return (
    <svg viewBox="0 0 256 256" fill="none" aria-hidden="true" className={className}>
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        fill="currentColor"
        d="M128 20L222 74V182L128 236L34 182V74L128 20ZM128 52L62 90V166L128 204L194 166V90L128 52Z"
      />
      <circle cx="128" cy="108" r="22" className="fill-success" />
      <path d="M116 122H140L148 160H108L116 122Z" className="fill-success" />
    </svg>
  )
}

/** Mark plus wordmark, as used in headers and the brand panel. */
export function Logo({ size = 'md' }: { size?: 'sm' | 'md' }) {
  const small = size === 'sm'
  return (
    <span className="flex items-center gap-2.5">
      <LogoMark className={small ? 'size-7' : 'size-9'} />
      <span className={small ? 'text-base font-semibold' : 'text-xl font-semibold'}>GitVault</span>
    </span>
  )
}
