import { useMutation, useQuery } from '@tanstack/react-query'

import { auditList, generate } from '@/api/tools'
import { queryKeys } from '@/config/query-keys'
import type { AuditQuery, GenerateInput } from '@/types'

export const useGenerate = () =>
  useMutation({ mutationFn: (input: GenerateInput) => generate(input) })

export const useAudit = (query: AuditQuery) =>
  useQuery({ queryKey: queryKeys.audit(query), queryFn: () => auditList(query) })
