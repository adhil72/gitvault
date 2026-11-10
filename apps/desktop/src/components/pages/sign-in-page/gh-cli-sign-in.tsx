import { useNavigate } from '@tanstack/react-router'
import { Terminal } from 'lucide-react'

import { ErrorNotice } from '@/components/general/error-notice'
import { Button } from '@/components/ui/button'
import { useLoginWithGh } from '@/hooks/use-auth'

/** One-click sign-in that reuses the GitHub CLI session (`gh auth login`), so no OAuth app is needed. */
export function GhCliSignIn() {
  const navigate = useNavigate()
  const login = useLoginWithGh()
  return (
    <div className="flex flex-col gap-3">
      <Button
        className="w-full"
        disabled={login.isPending}
        onClick={() =>
          login.mutate(undefined, { onSuccess: () => void navigate({ to: '/vaults' }) })
        }
      >
        <Terminal className="size-[18px]" />
        {login.isPending ? 'Signing in…' : 'Continue with GitHub CLI'}
      </Button>
      <ErrorNotice error={login.error} />
      <p className="text-xs text-muted-foreground">
        Uses your existing <code className="font-mono">gh</code> session. Not signed in yet? Run{' '}
        <code className="font-mono">gh auth login</code> in a terminal first.
      </p>
    </div>
  )
}
