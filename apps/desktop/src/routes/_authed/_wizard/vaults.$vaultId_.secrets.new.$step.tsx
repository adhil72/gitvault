import { createFileRoute, redirect } from '@tanstack/react-router'

import { AddSecretPage } from '@/components/pages/add-secret-page'
import { ADD_SECRET_STEPS, isStep } from '@/config/wizards'
import { vaultGuard } from '@/lib/vault-guard'

export const Route = createFileRoute('/_authed/_wizard/vaults/$vaultId_/secrets/new/$step')({
  beforeLoad: async ({ context, params }) => {
    await vaultGuard(context.queryClient, params.vaultId)
    if (!isStep(ADD_SECRET_STEPS, params.step)) {
      throw redirect({
        to: '/vaults/$vaultId/secrets/new/$step',
        params: { vaultId: params.vaultId, step: 'details' },
      })
    }
  },
  component: AddSecretPage,
})
