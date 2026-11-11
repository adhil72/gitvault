import { useNavigate } from '@tanstack/react-router'
import { useState, type FormEvent } from 'react'
import { toast } from 'sonner'

import { CopyButton } from '@/components/general/copy-button'
import { ErrorNotice } from '@/components/general/error-notice'
import { PageHeading } from '@/components/general/page-heading'
import { PasswordField } from '@/components/general/password-field'
import { Button } from '@/components/ui/button'
import { useChangePassword, useLockVault, useRegenerateRecovery } from '@/hooks/use-vaults'

import { SettingsCard, SettingsRow } from '../settings-row'
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
      <SettingsCard>
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
      </SettingsCard>
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
    <form
      onSubmit={submit}
      className="flex max-w-md flex-col gap-3 rounded-lg border border-border bg-card p-5"
    >
      <h2 className="font-semibold">Change master password</h2>
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
        disabled={!current || next.length < 8 || change.isPending}
      >
        Change password
      </Button>
    </form>
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
    <form
      onSubmit={submit}
      className="flex max-w-md flex-col gap-3 rounded-lg border border-border bg-card p-5"
    >
      <h2 className="font-semibold">Recovery key</h2>
      <p className="text-[13px] text-muted-foreground">
        Creates a new key. The old one stops working.
      </p>
      <PasswordField
        aria-label="Master password"
        placeholder="Master password"
        value={current}
        onChange={(e) => setCurrent(e.target.value)}
      />
      <ErrorNotice error={regenerate.error} />
      {key ? (
        <div className="flex items-center justify-between gap-2 rounded-md border border-input bg-inset p-3">
          <code className="font-mono text-sm break-all text-success">{key}</code>
          <CopyButton value={key} label="Copy recovery key" />
        </div>
      ) : null}
      <Button type="submit" variant="secondary" disabled={!current || regenerate.isPending}>
        Generate new recovery key
      </Button>
    </form>
  )
}
