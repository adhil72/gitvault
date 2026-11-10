import { Check, Copy } from 'lucide-react'
import { useEffect, useState } from 'react'

import { Button } from '@/components/ui/button'

const COPIED_FEEDBACK_MS = 2000

interface CopyButtonProps {
  value: string
  label?: string
}

/** Copies `value` to the clipboard and briefly shows a check mark. */
export function CopyButton({ value, label = 'Copy' }: CopyButtonProps) {
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    if (!copied) return
    const timer = setTimeout(() => setCopied(false), COPIED_FEEDBACK_MS)
    return () => clearTimeout(timer)
  }, [copied])

  async function handleCopy() {
    await navigator.clipboard.writeText(value)
    setCopied(true)
  }

  return (
    <Button type="button" variant="ghost" size="icon-sm" aria-label={label} onClick={handleCopy}>
      {copied ? <Check className="text-success" /> : <Copy />}
    </Button>
  )
}
