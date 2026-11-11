import { toast } from 'sonner'

import { ErrorNotice } from '@/components/general/error-notice'
import { Button } from '@/components/ui/button'
import { useDevices, useRevokeDevice, useTrustDevice } from '@/hooks/use-devices'
import { formatDate } from '@/lib/format'

import { SettingsGroup, SettingsRow } from '../settings-row'

const DEFAULT_NAME = 'This computer'

/** Devices that unlock this vault without the master password. */
export function TrustedDevicesCard({ vaultId }: { vaultId: string }) {
  const { data: devices = [] } = useDevices(vaultId)
  const trust = useTrustDevice(vaultId)
  const revoke = useRevokeDevice(vaultId)
  const trustedHere = devices.some((d) => d.this_device)
  return (
    <>
      <ErrorNotice error={trust.error ?? revoke.error} />
      <SettingsGroup
        title="Trusted devices"
        footer="A trusted device opens this vault without the master password. Revoke one to cut its access."
      >
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
        {trustedHere ? null : (
          <SettingsRow
            title="Trust this device"
            description="Skip the master password on this computer."
          >
            <Button
              variant="secondary"
              disabled={trust.isPending}
              onClick={() =>
                trust.mutate(DEFAULT_NAME, {
                  onSuccess: () => toast.success('This device is trusted'),
                })
              }
            >
              Trust
            </Button>
          </SettingsRow>
        )}
      </SettingsGroup>
    </>
  )
}
