import { useParams } from '@tanstack/react-router'
import { Search } from 'lucide-react'
import { useState } from 'react'

import { ErrorNotice } from '@/components/general/error-notice'
import { PageHeading } from '@/components/general/page-heading'
import { SelectField } from '@/components/general/select-field'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { useAudit } from '@/hooks/use-tools'
import { formatDate, timeAgo } from '@/lib/format'
import { cn } from '@/lib/utils'
import type { AuditKind } from '@/types'

const KIND_OPTIONS = [
  'created',
  'viewed',
  'copied',
  'updated',
  'rotated',
  'deleted',
  'restored',
  'imported',
  'exported',
  'synced',
  'unlocked',
  'locked',
].map((k) => ({ value: k, label: k[0]?.toUpperCase() + k.slice(1) }))

const TONES: Record<string, string> = {
  created: 'border-success/60 bg-success/10 text-success',
  synced: 'border-success/60 bg-success/10 text-success',
  viewed: 'border-info/60 bg-info/10 text-info',
  copied: 'border-info/60 bg-info/10 text-info',
  exported: 'border-info/60 bg-info/10 text-info',
  updated: 'border-warning/60 bg-warning/10 text-warning',
  imported: 'border-warning/60 bg-warning/10 text-warning',
  rotated: 'border-done/60 bg-done/10 text-done',
  restored: 'border-done/60 bg-done/10 text-done',
  deleted: 'border-destructive/60 bg-destructive/10 text-destructive',
}

export function AuditLogPage() {
  const { vaultId } = useParams({ from: '/_authed/vaults/$vaultId/audit' })
  const [search, setSearch] = useState('')
  const [kind, setKind] = useState('')
  const { data, isLoading, error } = useAudit({
    vault_id: vaultId,
    kind: (kind || null) as AuditKind | null,
    search: search || null,
    limit: 200,
  })

  return (
    <div className="flex flex-col gap-5 p-8">
      <PageHeading
        title="Audit log"
        description="Local activity on this device. Secret values are never recorded."
      />
      <div className="flex items-center gap-3">
        <div className="relative w-80">
          <Search className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search events"
            aria-label="Search events"
            className="h-9 pl-9"
          />
        </div>
        <div className="w-44">
          <SelectField
            id="audit-kind"
            value={kind}
            onChange={setKind}
            options={KIND_OPTIONS}
            allowEmpty
            placeholder="All events"
          />
        </div>
      </div>
      <ErrorNotice error={error} />
      {isLoading ? <Skeleton className="h-64 w-full" /> : null}
      <div className="rounded-lg border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Time</TableHead>
              <TableHead>Event</TableHead>
              <TableHead>Secret</TableHead>
              <TableHead>Device</TableHead>
              <TableHead>Detail</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.map((event, index) => (
              <TableRow key={`${event.at}-${index}`}>
                <TableCell className="text-muted-foreground" title={formatDate(event.at)}>
                  {timeAgo(event.at)}
                </TableCell>
                <TableCell>
                  <Badge
                    variant="outline"
                    className={cn('rounded-full capitalize', TONES[event.kind])}
                  >
                    {event.kind}
                  </Badge>
                </TableCell>
                <TableCell className="font-mono text-[13px] font-medium">
                  {event.secret_name ?? '—'}
                </TableCell>
                <TableCell className="text-muted-foreground">{event.device}</TableCell>
                <TableCell className="text-muted-foreground">{event.detail ?? ''}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
        {data?.length === 0 ? (
          <p className="p-6 text-center text-sm text-muted-foreground">No events yet.</p>
        ) : null}
      </div>
    </div>
  )
}
