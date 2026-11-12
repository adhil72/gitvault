import { useFacets } from '@/hooks/use-secrets'

const DEFAULT_ENVIRONMENTS = ['Production', 'Staging', 'Development']

/** The common environments plus every one already used in this vault. */
export function useEnvironmentOptions(vaultId: string): string[] {
  const { data: facets } = useFacets(vaultId)
  const used = facets?.environments.map((e) => e.name) ?? []
  return [...new Set([...DEFAULT_ENVIRONMENTS, ...used])]
}
