import { NoVaultShell } from '@/components/general/layouts/no-vault-shell'
import { NoVaultsState } from '@/components/general/no-vaults-state'
import { useAccount } from '@/hooks/use-auth'

/** Where an account lands when it has no vault to open, e.g. right after switching to it. */
export function NoVaultPage() {
  const { data: account } = useAccount()
  return (
    <NoVaultShell>
      <NoVaultsState login={account?.login} />
    </NoVaultShell>
  )
}
