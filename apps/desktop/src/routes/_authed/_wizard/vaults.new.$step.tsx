import { createFileRoute, redirect } from '@tanstack/react-router'

import { CreateVaultPage } from '@/components/pages/create-vault-page'
import { CREATE_VAULT_STEPS, isStep } from '@/config/wizards'

export const Route = createFileRoute('/_authed/_wizard/vaults/new/$step')({
  beforeLoad: ({ params }) => {
    if (!isStep(CREATE_VAULT_STEPS, params.step)) {
      throw redirect({ to: '/vaults/new/$step', params: { step: 'name' } })
    }
  },
  component: CreateVaultPage,
})
