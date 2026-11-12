import { Check } from 'lucide-react'
import { useEffect } from 'react'

import { Button } from '@/components/ui/button'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { useAddSecretNav } from '../use-add-secret-nav'

export function SavedStep() {
  const { go, backToVault } = useAddSecretNav()
  const { savedName, savedId, reset } = useAddSecretDraft()

  useEffect(() => {
    if (!savedId) go('details')
  }, [savedId, go])
  if (!savedId) return null

  return (
    <div className="flex flex-col items-center gap-6 py-6 text-center">
      <span className="flex size-16 items-center justify-center rounded-full border border-success/40 bg-success/10 ring-8 ring-success/5">
        <Check className="size-8 text-success" />
      </span>
      <header className="flex flex-col gap-1.5">
        <h1 className="text-2xl font-semibold tracking-[-0.02em]">Secret saved</h1>
        <p className="font-mono text-sm break-all text-foreground">{savedName}</p>
        <p className="mt-1 text-sm text-pretty text-muted-foreground">
          Encrypted, versioned and queued to sync with GitHub.
        </p>
      </header>
      <div className="flex w-full flex-col gap-2.5">
        <Button
          onClick={() => {
            reset()
            backToVault(savedId)
          }}
        >
          View secret
        </Button>
        <Button
          variant="secondary"
          onClick={() => {
            reset()
            go('details')
          }}
        >
          Add another secret
        </Button>
      </div>
    </div>
  )
}
