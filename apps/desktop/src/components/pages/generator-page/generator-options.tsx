import { Switch } from '@/components/ui/switch'
import type { GenerateInput } from '@/types'

interface GeneratorOptionsProps {
  options: GenerateInput
  onChange: (next: GenerateInput) => void
}

const TOGGLES: {
  key: 'uppercase' | 'lowercase' | 'digits' | 'symbols' | 'avoid_ambiguous'
  label: string
}[] = [
  { key: 'uppercase', label: 'Uppercase letters  A–Z' },
  { key: 'lowercase', label: 'Lowercase letters  a–z' },
  { key: 'digits', label: 'Numbers  0–9' },
  { key: 'symbols', label: 'Symbols  !@#$%' },
  { key: 'avoid_ambiguous', label: 'Avoid ambiguous characters  0 O l 1' },
]

function RangeRow({
  label,
  value,
  min,
  max,
  onChange,
}: {
  label: string
  value: number
  min: number
  max: number
  onChange: (v: number) => void
}) {
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <label htmlFor={`range-${label}`} className="text-[15px] font-semibold">
          {label}
        </label>
        <span className="rounded-md bg-muted px-2 py-0.5 font-mono text-[13px]">{value}</span>
      </div>
      <input
        id={`range-${label}`}
        type="range"
        min={min}
        max={max}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="accent-[var(--info)]"
      />
    </div>
  )
}

/** Controls that change with the generator mode (random, passphrase, PIN). */
export function GeneratorOptions({ options, onChange }: GeneratorOptionsProps) {
  if (options.kind === 'pin') {
    return (
      <RangeRow
        label="Length"
        value={options.length}
        min={4}
        max={12}
        onChange={(length) => onChange({ ...options, length })}
      />
    )
  }
  if (options.kind === 'passphrase') {
    return (
      <div className="flex flex-col gap-5">
        <RangeRow
          label="Words"
          value={options.words}
          min={3}
          max={12}
          onChange={(words) => onChange({ ...options, words })}
        />
        <div className="flex items-center justify-between text-sm">
          Capitalize each word
          <Switch
            checked={options.capitalize}
            onCheckedChange={(capitalize) => onChange({ ...options, capitalize })}
          />
        </div>
      </div>
    )
  }
  return (
    <div className="flex flex-col gap-5">
      <RangeRow
        label="Length"
        value={options.length}
        min={8}
        max={128}
        onChange={(length) => onChange({ ...options, length })}
      />
      {TOGGLES.map(({ key, label }) => (
        <div key={key} className="flex items-center justify-between text-sm">
          {label}
          <Switch
            checked={options[key]}
            onCheckedChange={(checked) => onChange({ ...options, [key]: checked })}
          />
        </div>
      ))}
    </div>
  )
}
