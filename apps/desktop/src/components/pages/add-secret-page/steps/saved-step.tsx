import { Check } from 'lucide-react'
import { useEffect } from 'react'

import { PageHeading } from '@/components/general/page-heading'
import { Button } from '@/components/ui/button'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { useAddSecretNav } from '../use-add-secret-nav'

export function SavedStep() {
  const { go, backToVault } = useAddSecretNav()
  const { savedName, savedId, reset } = useAddSecretDraft()

  useEffect(() => {
    if (!savedId) go('type')
  }, [savedId, go])
  if (!savedId) return null

  return (
    <div className="flex flex-col gap-6">
      <span className="flex size-16 items-center justify-center rounded-full border border-success bg-success/10">
        <Check className="size-8 text-success" />
      </span>
      <PageHeading
        title={`${savedName} saved`}
        description="Encrypted, versioned and queued to sync with GitHub."
      />
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
          go('type')
        }}
      >
        Add another secret
      </Button>
    </div>
  )
}
