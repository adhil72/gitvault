import { getRouteApi } from '@tanstack/react-router'
import { useState } from 'react'

import { FrameworkSwitcher } from '@/components/general/framework-switcher'
import { PageHeading } from '@/components/general/page-heading'
import { DocBlockView } from '@/components/pages/docs-page/doc-block'
import { CONNECT_TARGETS } from '@/config/connect-targets'
import { INTRO_DOCS, OUTRO_DOCS, type DocSection } from '@/config/integration-docs'

const route = getRouteApi('/_authed/vaults/$vaultId/docs/integration')

// The app uses hash routing, so plain #anchor links would change the route.
function scrollToSection(id: string) {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function Section({ section }: { section: DocSection }) {
  return (
    <section id={section.id} className="flex scroll-mt-6 flex-col gap-4">
      <h2 className="text-xl font-semibold tracking-tight">{section.title}</h2>
      {section.blocks.map((block, index) => (
        <DocBlockView
          key={index}
          block={block}
          label={`Copy ${section.title} sample ${index + 1}`}
        />
      ))}
    </section>
  )
}

function Toc({ sections }: { sections: readonly DocSection[] }) {
  return (
    <nav aria-label="On this page" className="sticky top-6 hidden h-fit w-44 shrink-0 lg:block">
      <p className="mb-2 text-xs font-medium tracking-wide text-muted-foreground uppercase">
        On this page
      </p>
      <ul className="flex flex-col gap-1.5 text-sm">
        {sections.map((section) => (
          <li key={section.id}>
            <button
              type="button"
              onClick={() => scrollToSection(section.id)}
              className="text-left text-muted-foreground transition-colors hover:text-foreground"
            >
              {section.title}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  )
}

/** Detailed guide to using a vault from code. The switcher at the top picks the framework section. */
export function DocsPage() {
  const { framework: initial } = route.useSearch()
  const [framework, setFramework] = useState(initial ?? CONNECT_TARGETS[0]?.id ?? '')
  const target = CONNECT_TARGETS.find((t) => t.id === framework)
  const guide: DocSection[] = target
    ? [{ id: 'framework-guide', title: target.label, blocks: target.guide }]
    : []
  const sections = [...INTRO_DOCS, ...guide, ...OUTRO_DOCS]

  return (
    <div className="mx-auto flex w-full max-w-5xl gap-10 px-6 py-8">
      <div className="flex min-w-0 flex-1 flex-col gap-10">
        <PageHeading
          title="Integration guide"
          description="Use this vault's secrets from your code without .env files. Values are decrypted on this device only."
        />
        <FrameworkSwitcher value={framework} onValueChange={setFramework} />
        {INTRO_DOCS.map((section) => (
          <Section key={section.id} section={section} />
        ))}
        {guide.map((section) => (
          <Section key={section.id} section={section} />
        ))}
        {OUTRO_DOCS.map((section) => (
          <Section key={section.id} section={section} />
        ))}
      </div>
      <Toc sections={sections} />
    </div>
  )
}
