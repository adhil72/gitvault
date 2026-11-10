import { useNavigate, useParams, useSearch } from '@tanstack/react-router'
import { Lock } from 'lucide-react'
import { useState, type FormEvent } from 'react'

import { ErrorNotice } from '@/components/general/error-notice'
import { FormField } from '@/components/general/form-field'
import { PageHeading } from '@/components/general/page-heading'
import { PasswordField } from '@/components/general/password-field'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useTrustDevice } from '@/hooks/use-devices'
import { useUnlockVault, useVault } from '@/hooks/use-vaults'

export function UnlockPage() {
  const { vaultId } = useParams({ from: '/_authed/_wizard/vaults/$vaultId_/unlock' })
  const { redirect } = useSearch({ from: '/_authed/_wizard/vaults/$vaultId_/unlock' })
  const navigate = useNavigate()
  const { data: vault } = useVault(vaultId)
  const unlock = useUnlockVault()
  const [secret, setSecret] = useState('')
  const [useRecovery, setUseRecovery] = useState(false)
  const [trustDevice, setTrustDevice] = useState(false)
  const trust = useTrustDevice(vaultId)

  const submit = (event: FormEvent) => {
    event.preventDefault()
    unlock.mutate(
      {
        vault_id: vaultId,
        password: useRecovery ? null : secret,
        recovery_key: useRecovery ? secret : null,
      },
      {
        onSuccess: () => {
          if (trustDevice) trust.mutate('This computer')
          void (redirect
            ? navigate({ href: redirect })
            : navigate({ to: '/vaults/$vaultId', params: { vaultId } }))
        },
      },
    )
  }

  return (
    <form onSubmit={submit} className="flex w-[360px] flex-col gap-6">
      <div className="flex items-center gap-3">
        <span className="flex size-10 items-center justify-center rounded-full bg-muted">
          <Lock className="size-5" />
        </span>
        <div>
          <p className="font-semibold">{vault?.name ?? 'Vault'}</p>
          <p className="text-xs text-muted-foreground">{vault?.repo ?? 'Local only'}</p>
        </div>
      </div>
      <PageHeading title="Unlock vault" />
      <FormField label={useRecovery ? 'Recovery key' : 'Master password'} htmlFor="unlock-secret">
        {useRecovery ? (
          <Input
            id="unlock-secret"
            autoFocus
            value={secret}
            onChange={(e) => setSecret(e.target.value)}
            placeholder="XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX"
            className="h-10 bg-inset font-mono"
          />
        ) : (
          <PasswordField
            id="unlock-secret"
            autoFocus
            value={secret}
            onChange={(e) => setSecret(e.target.value)}
            autoComplete="current-password"
          />
        )}
      </FormField>
      <div className="flex items-center gap-2.5">
        <Checkbox
          id="trust-device"
          checked={trustDevice}
          onCheckedChange={(checked) => setTrustDevice(checked === true)}
        />
        <Label htmlFor="trust-device" className="text-sm font-normal">
          Trust this device (unlock without the password next time)
        </Label>
      </div>
      <ErrorNotice error={unlock.error} />
      <Button type="submit" disabled={!secret || unlock.isPending}>
        {unlock.isPending ? 'Unlocking…' : 'Unlock vault'}
      </Button>
      <button
        type="button"
        className="text-sm text-info hover:underline"
        onClick={() => {
          setUseRecovery((v) => !v)
          setSecret('')
          unlock.reset()
        }}
      >
        {useRecovery ? 'Use master password' : 'Use recovery key instead'}
      </button>
    </form>
  )
}
