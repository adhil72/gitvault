import {
  CodeBlock,
  CodeBlockBody,
  CodeBlockContent,
  CodeBlockCopyButton,
  CodeBlockFilename,
  CodeBlockHeader,
  CodeBlockItem,
} from '@/components/ui/code-block'
import type { DocCode } from '@/config/connect-targets'

interface CodeSnippetProps extends DocCode {
  /** Accessible name of the copy button. */
  copyLabel: string
}

/** A highlighted, copyable code sample with its file name on top. */
export function CodeSnippet({ filename, language, code, copyLabel }: CodeSnippetProps) {
  const data = [{ language, filename, code }]
  return (
    <CodeBlock data={data} defaultValue={language}>
      <CodeBlockHeader>
        <CodeBlockFilename value={language}>{filename}</CodeBlockFilename>
        <CodeBlockCopyButton aria-label={copyLabel} className="ml-auto" />
      </CodeBlockHeader>
      <CodeBlockBody>
        {(item) => (
          <CodeBlockItem key={item.language} value={item.language} lineNumbers={false}>
            <CodeBlockContent language={language}>{item.code}</CodeBlockContent>
          </CodeBlockItem>
        )}
      </CodeBlockBody>
    </CodeBlock>
  )
}
