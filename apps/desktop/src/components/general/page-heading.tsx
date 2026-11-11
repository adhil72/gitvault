interface PageHeadingProps {
  title: string
  description?: string
}

/** Title (and optional description) at the top of a page or form. */
export function PageHeading({ title, description }: PageHeadingProps) {
  return (
    <header className="flex flex-col gap-1.5">
      <h1 className="text-[28px] leading-tight font-semibold tracking-[-0.02em]">{title}</h1>
      {description ? (
        <p className="text-[15px] text-pretty text-muted-foreground">{description}</p>
      ) : null}
    </header>
  )
}
