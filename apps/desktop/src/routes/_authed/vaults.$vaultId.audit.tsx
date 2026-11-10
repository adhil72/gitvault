import { createFileRoute } from '@tanstack/react-router'

import { AuditLogPage } from '@/components/pages/audit-log-page'

export const Route = createFileRoute('/_authed/vaults/$vaultId/audit')({ component: AuditLogPage })
