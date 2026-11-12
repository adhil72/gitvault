import { useNavigate, useParams } from '@tanstack/react-router'
import { RefreshCw } from 'lucide-react'
import { useEffect, useState } from 'react'

import { CopyButton } from '@/components/general/copy-button'
import { PageHeading } from '@/components/general/page-heading'
import { Button } from '@/components/ui/button'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useGenerate } from '@/hooks/use-tools'
import { useAddSecretDraft } from '@/stores/add-secret-draft'
import type { GenerateInput, GenerateKind, GeneratedDto } from '@/types'

import { GeneratorOptions } from './generator-options'

const DEFAULTS: GenerateInput = {
  kind: 'password',
  length: 20,
  uppercase: true,
  lowercase: true,
  digits: true,
  symbols: true,
  avoid_ambiguous: false,
  words: 5,
  separator: '-',
  capitalize: false,
}

const STRENGTH_TONES: Record<string, string> = {
  weak: 'text-destructive',
  fair: 'text-warning',
  strong: 'text-success',
  'very strong': 'text-success',
}

export function GeneratorPage() {
  const { vaultId } = useParams({ from: '/_authed/vaults/$vaultId/tools/generator' })
  const navigate = useNavigate()
  const [options, setOptions] = useState<GenerateInput>(DEFAULTS)
  const [result, setResult] = useState<GeneratedDto | null>(null)
  const [recent, setRecent] = useState<string[]>([])
  const generate = useGenerate()
  const run = () =>
    generate.mutate(options, {
      onSuccess: (r) => {
        setResult(r)
        setRecent((list) => [r.value, ...list].slice(0, 3))
      },
    })

  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(run, [JSON.stringify(options)])

  const saveAsSecret = () => {
    useAddSecretDraft.getState().update({ value: result?.value ?? '' })
    void navigate({
      to: '/vaults/$vaultId/secrets/new/$step',
      params: { vaultId, step: 'details' },
    })
  }

  return (
    <div className="flex flex-col gap-6 p-8">
      <PageHeading
        title="Password generator"
        description="Create strong passwords, passphrases and PINs. Nothing leaves this device."
      />
      <div className="grid max-w-5xl grid-cols-[1fr_460px] gap-6">
        <section className="flex flex-col gap-5 rounded-lg border border-border bg-card p-6">
          <Tabs
            value={options.kind}
            onValueChange={(kind) => setOptions({ ...options, kind: kind as GenerateKind })}
          >
            <TabsList>
              <TabsTrigger value="password">Random</TabsTrigger>
              <TabsTrigger value="passphrase">Passphrase</TabsTrigger>
              <TabsTrigger value="pin">PIN</TabsTrigger>
            </TabsList>
          </Tabs>
          <GeneratorOptions options={options} onChange={setOptions} />
        </section>
        <div className="flex flex-col gap-4">
          <section className="flex flex-col gap-4 rounded-lg border border-border bg-card p-6">
            <h2 className="text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">
              Generated value
            </h2>
            <div className="flex items-center gap-3 rounded-lg border border-input bg-inset px-4 py-4">
              <code
                className="min-w-0 flex-1 break-all font-mono text-xl font-medium text-success"
                data-testid="generated-value"
              >
                {result?.value ?? '…'}
              </code>
              <Button variant="ghost" size="icon-sm" aria-label="Generate again" onClick={run}>
                <RefreshCw />
              </Button>
            </div>
            {result ? (
              <p className={`text-xs font-medium ${STRENGTH_TONES[result.strength] ?? ''}`}>
                {result.strength} · {Math.round(result.entropy_bits)} bits
              </p>
            ) : null}
            <div className="flex gap-3">
              <CopyValueButton value={result?.value ?? ''} />
              <Button
                variant="secondary"
                className="flex-1"
                disabled={!result}
                onClick={saveAsSecret}
              >
                Save as secret
              </Button>
            </div>
          </section>
          <section className="flex flex-col gap-2 rounded-lg border border-border bg-card p-5">
            <h2 className="text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">
              Recent (this session)
            </h2>
            {recent.map((value) => (
              <code key={value} className="truncate font-mono text-[13px] text-muted-foreground">
                {value}
              </code>
            ))}
          </section>
        </div>
      </div>
    </div>
  )
}

function CopyValueButton({ value }: { value: string }) {
  return (
    <span className="flex flex-1 items-center justify-center gap-2 rounded-md bg-primary px-4 text-sm font-semibold text-primary-foreground">
      <CopyButton value={value} label="Copy generated value" />
      Copy
    </span>
  )
}
