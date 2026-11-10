import { toast } from 'sonner'

import { ErrorNotice } from '@/components/general/error-notice'
import { PageHeading } from '@/components/general/page-heading'
import { SelectField } from '@/components/general/select-field'
import { Switch } from '@/components/ui/switch'
import { useSettings, useUpdateSettings } from '@/hooks/use-settings'
import type { Settings } from '@/types'

import { SettingsCard, SettingsRow } from '../settings-row'

const MINUTES = [
  ['1', '1 minute'],
  ['5', '5 minutes'],
  ['15', '15 minutes'],
  ['30', '30 minutes'],
  ['60', '1 hour'],
  ['0', 'Never'],
]
const SECONDS = [
  ['15', '15 seconds'],
  ['30', '30 seconds'],
  ['60', '1 minute'],
  ['0', 'Never'],
]
const toOptions = (pairs: string[][]) =>
  pairs.map(([value, label]) => ({ value: value ?? '', label: label ?? '' }))

export function GeneralTab() {
  const { data: settings } = useSettings()
  const update = useUpdateSettings()
  if (!settings) return null
  const save = (patch: Partial<Settings>) =>
    update.mutate({ ...settings, ...patch }, { onSuccess: () => toast.success('Settings saved') })

  return (
    <>
      <PageHeading title="General" description="Locking, clipboard and sync behavior." />
      <ErrorNotice error={update.error} />
      <SettingsCard>
        <SettingsRow title="Auto-lock vault" description="Lock after a period of inactivity.">
          <div className="w-44">
            <SelectField
              id="auto-lock"
              value={String(settings.auto_lock_minutes)}
              onChange={(v) => save({ auto_lock_minutes: Number(v) })}
              options={toOptions(MINUTES)}
            />
          </div>
        </SettingsRow>
        <SettingsRow
          title="Clear clipboard"
          description="Remove copied secrets from the clipboard after a delay."
        >
          <div className="w-44">
            <SelectField
              id="clipboard"
              value={String(settings.clear_clipboard_seconds)}
              onChange={(v) => save({ clear_clipboard_seconds: Number(v) })}
              options={toOptions(SECONDS)}
            />
          </div>
        </SettingsRow>
        <SettingsRow
          title="Hide revealed values"
          description="Mask a revealed value again after a delay."
        >
          <div className="w-44">
            <SelectField
              id="hide"
              value={String(settings.hide_revealed_seconds)}
              onChange={(v) => save({ hide_revealed_seconds: Number(v) })}
              options={toOptions(SECONDS)}
            />
          </div>
        </SettingsRow>
        <SettingsRow
          title="Auto-sync"
          description="Push encrypted changes to GitHub after every edit."
        >
          <Switch
            aria-label="Auto-sync"
            checked={settings.auto_sync}
            onCheckedChange={(auto_sync) => save({ auto_sync })}
          />
        </SettingsRow>
      </SettingsCard>
    </>
  )
}
