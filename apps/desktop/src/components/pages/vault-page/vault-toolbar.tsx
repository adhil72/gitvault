import { Upload } from 'lucide-react'

import { SelectField } from '@/components/general/select-field'
import { Button } from '@/components/ui/button'
import type { VaultSearch } from '@/schemas/vault-search'

import { titleFor } from './build-query'

const SORTS = [
  { value: 'updated', label: 'Recently updated' },
  { value: 'name', label: 'Name' },
  { value: 'expiry', label: 'Expiry date' },
]

interface VaultToolbarProps {
  count: number
  search: VaultSearch
  onSort: (sort: VaultSearch['sort']) => void
  onImport: () => void
}

export function VaultToolbar({ count, search, onSort, onImport }: VaultToolbarProps) {
  return (
    <div className="flex items-center justify-between gap-4 px-8 pt-7 pb-4">
      <div className="flex items-baseline gap-3">
        <h1 className="text-[28px] leading-tight font-semibold tracking-[-0.02em]">
          {titleFor(search)}
        </h1>
        <span className="text-sm text-muted-foreground tabular-nums">
          {count} secret{count === 1 ? '' : 's'}
        </span>
      </div>
      <div className="flex items-center gap-2.5">
        <div className="w-44">
          <SelectField
            id="sort"
            value={search.sort ?? 'updated'}
            onChange={(v) => onSort(v as VaultSearch['sort'])}
            options={SORTS}
          />
        </div>
        <Button variant="secondary" onClick={onImport}>
          <Upload /> Import
        </Button>
      </div>
    </div>
  )
}
