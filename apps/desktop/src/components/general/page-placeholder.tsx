interface PagePlaceholderProps {
  title: string
  description?: string
}

/** Temporary stand-in until a page is built from its Figma screen. */
export function PagePlaceholder({ title, description }: PagePlaceholderProps) {
  return (
    <section className="flex flex-col gap-2 p-8">
      <h1 className="text-2xl font-semibold">{title}</h1>
      {description ? <p className="text-sm opacity-70">{description}</p> : null}
    </section>
  )
}
