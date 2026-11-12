import { useNavigate } from '@tanstack/react-router'
import { Database } from 'lucide-react'
import { toast } from 'sonner'

import { GithubMark } from '@/components/general/github-mark'
import { Button } from '@/components/ui/button'
import { useConnectVault, useRemoteVaults } from '@/hooks/use-vaults'

/** Vaults that already exist in the account's GitHub but are not on this device yet. */
export function GithubVaultList() {
  const navigate = useNavigate()
  const connect = useConnectVault()
  const { data } = useRemoteVaults(true)
  const repos = (data?.vaults ?? []).filter((vault) => !vault.id && vault.repo)
  if (repos.length === 0) return null

  const connectRepo = (repo: string) =>
    connect.mutate(repo, {
      onSuccess: (vault) =>
        void navigate({ to: '/vaults/$vaultId', params: { vaultId: vault.id ?? '' } }),
      onError: (error) => toast.error(error.message),
    })

  return (
    <section className="flex w-full max-w-md flex-col gap-2">
      <h3 className="flex items-center gap-2 text-xs font-medium tracking-wide text-muted-foreground uppercase">
        <GithubMark className="size-3.5" /> Found on GitHub
      </h3>
      <ul className="flex flex-col gap-2">
        {repos.map((vault) => (
          <li
            key={vault.repo}
            className="flex items-center gap-3 rounded-lg border border-border bg-card p-3"
          >
            <Database className="size-4 text-muted-foreground" />
            <span className="min-w-0 flex-1 truncate font-mono text-sm">{vault.repo}</span>
            <Button
              size="sm"
              disabled={connect.isPending}
              onClick={() => connectRepo(vault.repo ?? '')}
            >
              Connect
            </Button>
          </li>
        ))}
      </ul>
    </section>
  )
}
