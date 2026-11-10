/** A GitHub-safe repository name derived from free text (mirrors the backend rule). */
export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 80)
}

/** Default repository name for a vault: `gitvault-<name>`. */
export function defaultRepoName(vaultName: string): string {
  return slugify(`gitvault-${vaultName}`)
}
