import { Button } from '@/components/ui/button'
import { useAppVersion, useInstallUpdate, useUpdateCheck } from '@/hooks/use-updater'

import { SettingsRow } from '../settings-row'

/** "Check for updates" row. Updates are signature-verified by the app before install. */
export function UpdatesRow() {
  const { data: version } = useAppVersion()
  const check = useUpdateCheck()
  const install = useInstallUpdate()
  const update = check.data

  const status = (() => {
    if (check.isError) return 'Could not reach the update server.'
    if (update) return `Version ${update.version} is available.`
    if (check.isFetched) return 'You are on the latest version.'
    return version ? `Current version ${version}.` : 'Updates are signed and verified.'
  })()

  return (
    <SettingsRow title="Updates" description={status}>
      {update ? (
        <Button onClick={() => install.mutate()} disabled={install.isPending}>
          {install.isPending ? 'Installing…' : 'Install and restart'}
        </Button>
      ) : (
        <Button variant="outline" onClick={() => void check.refetch()} disabled={check.isFetching}>
          {check.isFetching ? 'Checking…' : 'Check for updates'}
        </Button>
      )}
    </SettingsRow>
  )
}
