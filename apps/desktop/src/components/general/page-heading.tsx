interface PageHeadingProps {
  title: string
  description?: string
}

/** Title (and optional description) at the top of a page or form. */
export function PageHeading({ title, description }: PageHeadingProps) {
  return (
    <header className="flex flex-col gap-2">
      <h1 className="text-[28px] leading-tight font-bold">{title}</h1>
      {description ? <p className="text-[15px] text-muted-foreground">{description}</p> : null}
    </header>
  )
}
