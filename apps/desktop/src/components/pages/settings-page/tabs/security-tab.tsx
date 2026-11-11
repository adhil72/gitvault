import { useNavigate } from '@tanstack/react-router'
import { useState, type FormEvent } from 'react'
import { toast } from 'sonner'

import { CopyButton } from '@/components/general/copy-button'
import { ErrorNotice } from '@/components/general/error-notice'
import { PageHeading } from '@/components/general/page-heading'
import { PasswordField } from '@/components/general/password-field'
import { Button } from '@/components/ui/button'
import { useChangePassword, useLockVault, useRegenerateRecovery } from '@/hooks/use-vaults'

import { SettingsGroup, SettingsRow } from '../settings-row'
import { TrustedDevicesCard } from './trusted-devices-card'

export function SecurityTab({ vaultId }: { vaultId: string }) {
  const navigate = useNavigate()
  const lock = useLockVault()
  return (
    <>
      <PageHeading title="Security" description="Master password, recovery key and locking." />
      <ChangePasswordForm vaultId={vaultId} />
      <RecoveryKeyForm vaultId={vaultId} />
      <TrustedDevicesCard vaultId={vaultId} />
      <SettingsGroup title="Session">
        <SettingsRow
          title="Lock vault now"
          description="Wipe the key from memory and require the master password again."
        >
          <Button
            variant="danger"
            onClick={() =>
              lock.mutate(vaultId, {
                onSuccess: () => void navigate({ to: '/vaults/$vaultId', params: { vaultId } }),
              })
            }
          >
            Lock now
          </Button>
        </SettingsRow>
      </SettingsGroup>
    </>
  )
}

function ChangePasswordForm({ vaultId }: { vaultId: string }) {
  const change = useChangePassword()
  const [current, setCurrent] = useState('')
  const [next, setNext] = useState('')
  const submit = (event: FormEvent) => {
    event.preventDefault()
    change.mutate(
      { vaultId, current, newPassword: next },
      {
        onSuccess: () => {
          setCurrent('')
          setNext('')
          toast.success('Master password changed')
        },
      },
    )
  }
  return (
    <SettingsGroup title="Master password">
      <form onSubmit={submit} className="flex flex-col gap-3 p-5">
        <PasswordField
          aria-label="Current master password"
          placeholder="Current password"
          value={current}
          onChange={(e) => setCurrent(e.target.value)}
          autoComplete="current-password"
        />
        <PasswordField
          aria-label="New master password"
          placeholder="New password (8+ characters)"
          value={next}
          onChange={(e) => setNext(e.target.value)}
          autoComplete="new-password"
        />
        <ErrorNotice error={change.error} />
        <Button
          type="submit"
          variant="secondary"
          className="self-end"
          disabled={!current || next.length < 8 || change.isPending}
        >
          Change password
        </Button>
      </form>
    </SettingsGroup>
  )
}

function RecoveryKeyForm({ vaultId }: { vaultId: string }) {
  const regenerate = useRegenerateRecovery()
  const [current, setCurrent] = useState('')
  const [key, setKey] = useState<string | null>(null)
  const submit = (event: FormEvent) => {
    event.preventDefault()
    regenerate.mutate(
      { vaultId, current },
      {
        onSuccess: (next) => {
          setKey(next)
          setCurrent('')
        },
      },
    )
  }
  return (
    <SettingsGroup title="Recovery key" footer="Creates a new key. The old one stops working.">
      <form onSubmit={submit} className="flex flex-col gap-3 p-5">
        <PasswordField
          aria-label="Master password"
          placeholder="Master password"
          value={current}
          onChange={(e) => setCurrent(e.target.value)}
        />
        <ErrorNotice error={regenerate.error} />
        {key ? (
          <div className="flex items-center justify-between gap-2 rounded-lg bg-inset p-3 ring-1 ring-border/50">
            <code className="font-mono text-sm break-all text-success">{key}</code>
            <CopyButton value={key} label="Copy recovery key" />
          </div>
        ) : null}
        <Button
          type="submit"
          variant="secondary"
          className="self-end"
          disabled={!current || regenerate.isPending}
        >
          Generate new recovery key
        </Button>
      </form>
    </SettingsGroup>
  )
}
