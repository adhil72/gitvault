import { useNavigate, useParams } from '@tanstack/react-router'

import type { AddSecretStep } from '@/config/wizards'

/** Vault id and step navigation for the add-secret wizard. */
export function useAddSecretNav() {
  const navigate = useNavigate()
  const { vaultId } = useParams({ from: '/_authed/_wizard/vaults/$vaultId_/secrets/new/$step' })
  const go = (step: AddSecretStep) =>
    void navigate({ to: '/vaults/$vaultId/secrets/new/$step', params: { vaultId, step } })
  const backToVault = (secretId?: string) =>
    void navigate({
      to: '/vaults/$vaultId',
      params: { vaultId },
      search: secretId ? { secret: secretId } : {},
    })
  return { vaultId, go, backToVault }
}
