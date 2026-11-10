import { useNavigate } from '@tanstack/react-router'
import { Download } from 'lucide-react'
import { useEffect, useState } from 'react'

import { Callout } from '@/components/general/callout'
import { CopyButton } from '@/components/general/copy-button'
import { PageHeading } from '@/components/general/page-heading'
import { WizardActions } from '@/components/general/wizard-actions'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { useCreateVaultDraft } from '@/stores/create-vault-draft'

function downloadText(filename: string, text: string) {
  const url = URL.createObjectURL(new Blob([text], { type: 'text/plain' }))
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)
}

export function RecoveryStep() {
  const navigate = useNavigate()
  const result = useCreateVaultDraft((s) => s.result)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    if (!result) void navigate({ to: '/vaults/new/$step', params: { step: 'name' }, replace: true })
  }, [result, navigate])
  if (!result) return null

  const groups = result.recoveryKey.split('-')
  return (
    <div className="flex flex-col gap-5">
      <PageHeading
        title="Save your recovery key"
        description="Print it or write it down. We will not show it again."
      />
      <div className="flex items-start justify-between gap-3 rounded-lg border border-input bg-inset p-5">
        <div
          className="grid grid-cols-4 gap-x-4 gap-y-2 font-mono text-lg font-medium text-success"
          data-testid="recovery-key"
        >
          {groups.map((group, index) => (
            <span key={`${group}-${index}`}>{group}</span>
          ))}
        </div>
        <CopyButton value={result.recoveryKey} label="Copy recovery key" />
      </div>
      <Button
        variant="secondary"
        onClick={() =>
          downloadText(
            'gitvault-recovery-key.txt',
            `GitVault recovery key for ${result.vaultName}\n\n${result.recoveryKey}\n`,
          )
        }
      >
        <Download /> Download as text file
      </Button>
      <label className="flex items-center gap-2.5 text-sm">
        <Checkbox checked={saved} onCheckedChange={(v) => setSaved(v === true)} />I have stored my
        recovery key somewhere safe
      </label>
      {result.remoteError ? (
        <Callout tone="warning">
          The vault was created on this computer, but pushing to GitHub failed: {result.remoteError}
        </Callout>
      ) : null}
      <WizardActions
        nextType="button"
        nextDisabled={!saved}
        nextLabel="Finish setup"
        onNext={() => void navigate({ to: '/vaults/new/$step', params: { step: 'done' } })}
        backLabel="Back"
        onBack={() => void navigate({ to: '/vaults/new/$step', params: { step: 'password' } })}
      />
    </div>
  )
}
