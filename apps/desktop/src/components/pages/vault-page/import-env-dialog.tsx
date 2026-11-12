import { useEffect, useState, type ChangeEvent } from 'react'
import { toast } from 'sonner'

import { EnvironmentField } from '@/components/general/environment-field'
import { ErrorNotice } from '@/components/general/error-notice'
import { FormField } from '@/components/general/form-field'
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
import { Textarea } from '@/components/ui/textarea'
import { useImportEnv, useImportPreview } from '@/hooks/use-secrets'

import type { useVaultNav } from './use-vault-nav'

/** Paste or choose a .env file, preview what will happen, then import it as secrets. */
export function ImportEnvDialog({ nav }: { nav: ReturnType<typeof useVaultNav> }) {
  const open = nav.search.panel === 'import'
  const [content, setContent] = useState('')
  const [environment, setEnvironment] = useState('')
  const [overwrite, setOverwrite] = useState(false)
  const preview = useImportPreview()
  const run = useImportEnv(nav.vaultId)
  const input = {
    vault_id: nav.vaultId,
    content,
    environment: environment || null,
    overwrite,
  }

  useEffect(() => {
    if (open && content.trim()) preview.mutate(input)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, content, environment])

  const readFile = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) setContent(await file.text())
  }

  const submit = () =>
    run.mutate(input, {
      onSuccess: (report) => {
        toast.success(
          `Imported: ${report.created} created, ${report.updated} updated, ${report.skipped} skipped`,
        )
        setContent('')
        nav.closePanel()
      },
    })

  return (
    <Dialog open={open} onOpenChange={(next) => !next && nav.closePanel()}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>Import from .env</DialogTitle>
          <DialogDescription>
            Each variable becomes a secret. Existing names are skipped unless you overwrite them.
          </DialogDescription>
        </DialogHeader>
        <input
          type="file"
          accept=".env,text/plain"
          onChange={readFile}
          aria-label="Choose a .env file"
          className="text-sm"
        />
        <Textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder={'STRIPE_KEY=sk_live_…\nDATABASE_URL=postgres://…'}
          spellCheck={false}
          className="min-h-28 bg-inset font-mono text-[13px]"
          aria-label=".env contents"
        />
        <div className="grid grid-cols-2 gap-3">
          <FormField label="Environment" htmlFor="imp-env">
            <EnvironmentField
              id="imp-env"
              value={environment}
              onChange={setEnvironment}
              vaultId={nav.vaultId}
            />
          </FormField>
        </div>
        {preview.data?.length ? (
          <ul
            className="max-h-40 overflow-y-auto rounded-md border border-border text-xs"
            aria-label="Preview"
          >
            {preview.data.map((entry) => (
              <li
                key={entry.name}
                className="flex items-center gap-3 border-b border-border px-3 py-2 last:border-b-0"
              >
                <span className="flex-1 truncate font-mono">{entry.name}</span>
                <span className="font-mono text-muted-foreground">{entry.masked_value}</span>
                <span className={entry.status === 'new' ? 'text-success' : 'text-warning'}>
                  {entry.status === 'new' ? 'New' : 'Duplicate'}
                </span>
              </li>
            ))}
          </ul>
        ) : null}
        <label className="flex items-center gap-2 text-sm">
          <Checkbox checked={overwrite} onCheckedChange={(v) => setOverwrite(v === true)} />{' '}
          Overwrite existing secrets
        </label>
        <ErrorNotice error={run.error ?? preview.error} />
        <DialogFooter>
          <Button variant="ghost" onClick={nav.closePanel}>
            Cancel
          </Button>
          <Button disabled={!preview.data?.length || run.isPending} onClick={submit}>
            {run.isPending ? 'Importing…' : `Import ${preview.data?.length ?? 0} secrets`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
