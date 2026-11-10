import { z } from 'zod'

/** URL search parameters of the vault page (filters, selection and open panel). */
export const vaultSearchSchema = z.object({
  q: z.string().optional(),
  environment: z.string().optional(),
  tag: z.string().optional(),
  view: z.enum(['favorites', 'expiring']).optional(),
  sort: z.enum(['updated', 'name', 'expiry']).optional(),
  secret: z.string().optional(),
  panel: z.enum(['edit', 'delete', 'import']).optional(),
})

export type VaultSearch = z.infer<typeof vaultSearchSchema>
