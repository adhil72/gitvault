import { useNavigate } from '@tanstack/react-router'
import { Plus, Vault } from 'lucide-react'

import { GithubVaultList } from '@/components/general/github-vault-list'
import { Button } from '@/components/ui/button'

/** Shown when the signed-in GitHub account has no vault yet, e.g. right after switching accounts. */
export function NoVaultsState({ login }: { login: string | undefined }) {
  const navigate = useNavigate()
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-8 p-10 pb-20">
      <div className="flex max-w-md animate-rise flex-col items-center gap-2 text-center motion-reduce:animate-none">
        <span className="mb-4 flex size-16 items-center justify-center rounded-[20px] bg-gradient-to-b from-success/15 to-success/5 shadow-[0_8px_32px_-8px] shadow-success/30 ring-1 ring-success/20 ring-inset">
          <Vault className="size-7 text-success" strokeWidth={1.75} />
        </span>
        <h2 className="text-[22px] font-semibold tracking-[-0.02em]">
          {login ? `Welcome, ${login}` : 'Welcome'}
        </h2>
        <p className="text-[15px] leading-relaxed text-pretty text-muted-foreground">
          You don&apos;t have a vault on this account yet. Create one to keep secrets in a private
          GitHub repository, or switch to another account from the menu above.
        </p>
        <Button
          className="mt-4 h-10 rounded-lg px-5"
          onClick={() => void navigate({ to: '/vaults/new/$step', params: { step: 'name' } })}
        >
          <Plus /> Create a vault
        </Button>
      </div>
      <GithubVaultList />
    </div>
  )
}
