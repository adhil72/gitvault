import { Lock } from 'lucide-react'
import { useState, type FormEvent } from 'react'

import { ErrorNotice } from '@/components/general/error-notice'
import { FormField } from '@/components/general/form-field'
import { PasswordField } from '@/components/general/password-field'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useTrustDevice } from '@/hooks/use-devices'
import { useUnlockVault, useVault } from '@/hooks/use-vaults'
import { useUnlockRequest } from '@/stores/unlock-request'

/** Asks for the master password (or recovery key) when something needs the vault key. */
export function UnlockDialog({ vaultId }: { vaultId: string }) {
  const requested = useUnlockRequest((s) => s.vaultId)
  const close = useUnlockRequest((s) => s.close)
  return (
    <Dialog open={requested === vaultId} onOpenChange={(open) => !open && close()}>
      <DialogContent className="sm:max-w-md">
        {requested === vaultId ? <UnlockForm vaultId={vaultId} /> : null}
      </DialogContent>
    </Dialog>
  )
}

function UnlockForm({ vaultId }: { vaultId: string }) {
  const { data: vault } = useVault(vaultId)
  const unlock = useUnlockVault()
  const trust = useTrustDevice(vaultId)
  const [secret, setSecret] = useState('')
  const [useRecovery, setUseRecovery] = useState(false)
  const [trustDevice, setTrustDevice] = useState(false)

  const submit = (event: FormEvent) => {
    event.preventDefault()
    const input = {
      vault_id: vaultId,
      password: useRecovery ? null : secret,
      recovery_key: useRecovery ? secret : null,
    }
    unlock.mutate(input, {
      onSuccess: () => {
        if (trustDevice) trust.mutate('This computer')
        const { then, close } = useUnlockRequest.getState()
        close()
        then?.()
      },
    })
  }

  const toggleRecovery = () => {
    setUseRecovery((v) => !v)
    setSecret('')
    unlock.reset()
  }

  return (
    <form onSubmit={submit} className="flex flex-col gap-5">
      <DialogHeader>
        <DialogTitle className="flex items-center gap-2">
          <Lock className="size-4" /> Unlock {vault?.name ?? 'vault'}
        </DialogTitle>
        <DialogDescription>
          Enter the master password to view, copy or change secrets.
        </DialogDescription>
      </DialogHeader>
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
      <DialogFooter className="items-center sm:justify-between">
        <button
          type="button"
          className="text-sm text-info hover:underline"
          onClick={toggleRecovery}
        >
          {useRecovery ? 'Use master password' : 'Use recovery key instead'}
        </button>
        <Button type="submit" disabled={!secret || unlock.isPending}>
          {unlock.isPending ? 'Unlocking…' : 'Unlock vault'}
        </Button>
      </DialogFooter>
    </form>
  )
}
