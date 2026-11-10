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
    <div className="flex items-center justify-between gap-4 px-6 py-5">
      <div className="flex items-center gap-3">
        <h1 className="text-2xl font-bold">{titleFor(search)}</h1>
        <span className="rounded-full bg-muted px-2.5 py-0.5 text-xs font-medium text-muted-foreground">
          {count} secret{count === 1 ? '' : 's'}
        </span>
      </div>
      <div className="flex items-center gap-2.5">
        <div className="w-48">
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
