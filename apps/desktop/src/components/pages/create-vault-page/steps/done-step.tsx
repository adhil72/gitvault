import { useNavigate } from '@tanstack/react-router'
import { Check } from 'lucide-react'
import { useEffect } from 'react'

import { PageHeading } from '@/components/general/page-heading'
import { Button } from '@/components/ui/button'
import { useCreateVaultDraft } from '@/stores/create-vault-draft'

export function DoneStep() {
  const navigate = useNavigate()
  const result = useCreateVaultDraft((s) => s.result)
  const reset = useCreateVaultDraft((s) => s.reset)

  useEffect(() => {
    if (!result) void navigate({ to: '/vaults/new/$step', params: { step: 'name' }, replace: true })
  }, [result, navigate])
  if (!result) return null

  const rows: [string, string][] = [
    ['Vault', result.vaultName],
    ['Repository', result.repo ?? 'Local only'],
    ['Encryption', 'AES-256-GCM'],
    ['Recovery key', 'Saved by you'],
  ]
  const open = () => {
    const vaultId = result.vaultId
    reset()
    void navigate({ to: '/vaults/$vaultId', params: { vaultId } })
  }

  return (
    <div className="flex flex-col gap-6">
      <span className="flex size-16 items-center justify-center rounded-full border border-success bg-success/10">
        <Check className="size-8 text-success" />
      </span>
      <PageHeading
        title={`${result.vaultName} is ready`}
        description="Your vault was created and is unlocked."
      />
      <dl className="flex flex-col rounded-lg border border-border bg-card">
        {rows.map(([label, value], index) => (
          <div
            key={label}
            className={`flex gap-3 px-4 py-3 text-sm ${index < rows.length - 1 ? 'border-b border-border' : ''}`}
          >
            <dt className="w-28 text-muted-foreground">{label}</dt>
            <dd className="font-medium">{value}</dd>
          </div>
        ))}
      </dl>
      <Button onClick={open}>Open vault</Button>
    </div>
  )
}
