import { toast } from 'sonner'

import { ErrorNotice } from '@/components/general/error-notice'
import { Button } from '@/components/ui/button'
import { useDevices, useRevokeDevice, useTrustDevice } from '@/hooks/use-devices'
import { formatDate } from '@/lib/format'

import { SettingsCard, SettingsRow } from '../settings-row'

const DEFAULT_NAME = 'This computer'

/** Devices that unlock this vault without the master password. */
export function TrustedDevicesCard({ vaultId }: { vaultId: string }) {
  const { data: devices = [] } = useDevices(vaultId)
  const trust = useTrustDevice(vaultId)
  const revoke = useRevokeDevice(vaultId)
  const trustedHere = devices.some((d) => d.this_device)
  return (
    <SettingsCard>
      <SettingsRow
        title="Trusted devices"
        description="A trusted device opens this vault without the master password. Revoke one to cut its access."
      >
        {trustedHere ? null : (
          <Button
            variant="secondary"
            disabled={trust.isPending}
            onClick={() =>
              trust.mutate(DEFAULT_NAME, {
                onSuccess: () => toast.success('This device is trusted'),
              })
            }
          >
            Trust this device
          </Button>
        )}
      </SettingsRow>
      <ErrorNotice error={trust.error ?? revoke.error} />
      {devices.map((device) => (
        <SettingsRow
          key={device.id}
          title={device.this_device ? `${device.name} (this device)` : device.name}
          description={`Trusted ${formatDate(device.created_at)}`}
        >
          <Button
            variant="danger"
            disabled={revoke.isPending}
            onClick={() => revoke.mutate(device.id)}
          >
            Revoke
          </Button>
        </SettingsRow>
      ))}
    </SettingsCard>
  )
}
