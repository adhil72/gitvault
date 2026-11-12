import { Info } from 'lucide-react'

import { CodeSnippet } from '@/components/general/code-snippet'
import type { DocBlock } from '@/config/connect-targets'

/** Renders one block of a docs section. */
export function DocBlockView({ block, label }: { block: DocBlock; label: string }) {
  switch (block.kind) {
    case 'text':
      return <p className="text-pretty text-muted-foreground">{block.text}</p>
    case 'list':
      return (
        <ul className="flex list-disc flex-col gap-1.5 pl-5 text-muted-foreground marker:text-muted-foreground/50">
          {block.items.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      )
    case 'note':
      return (
        <p className="flex gap-2 rounded-lg border border-warning/40 bg-warning/10 px-3 py-2 text-sm">
          <Info className="mt-0.5 size-4 shrink-0 text-warning" />
          {block.text}
        </p>
      )
    case 'code':
      return <CodeSnippet {...block} copyLabel={label} />
  }
}
