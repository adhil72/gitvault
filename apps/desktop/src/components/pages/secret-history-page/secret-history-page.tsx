import { Link, useParams } from '@tanstack/react-router'
import { ArrowLeft } from 'lucide-react'
import { toast } from 'sonner'

import { ErrorNotice } from '@/components/general/error-notice'
import { GithubMark } from '@/components/general/github-mark'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useHistory, useRestoreSecret, useSecret } from '@/hooks/use-secrets'
import { timeAgo } from '@/lib/format'
import { cn } from '@/lib/utils'

const SHORT_COMMIT = 7

export function SecretHistoryPage() {
  const { vaultId, secretId } = useParams({ from: '/_authed/vaults/$vaultId/history/$secretId' })
  const { data: secret } = useSecret(vaultId, secretId)
  const { data: history, isLoading, error } = useHistory(vaultId, secretId)
  const restore = useRestoreSecret(vaultId)

  return (
    <div className="mx-auto flex w-full max-w-3xl flex-col gap-5 p-8">
      <Button asChild variant="ghost" size="sm" className="self-start">
        <Link to="/vaults/$vaultId" params={{ vaultId }} search={{ secret: secretId }}>
          <ArrowLeft /> Back to secret
        </Link>
      </Button>
      <header className="flex flex-col gap-1.5">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-bold">Version history</h1>
          {secret ? (
            <span className="rounded-md bg-muted px-2.5 py-1 font-mono text-xs font-medium">
              {secret.name}
            </span>
          ) : null}
        </div>
        <p className="text-sm text-muted-foreground">
          Every change is a commit in your vault repository. Values are never shown here.
        </p>
      </header>
      <ErrorNotice error={error ?? restore.error} />
      {isLoading ? <Skeleton className="h-40 w-full" /> : null}
      <ol className="flex flex-col gap-2.5">
        {history?.map((entry, index) => (
          <li
            key={entry.commit}
            className={cn(
              'flex items-center gap-4 rounded-lg border p-4',
              index === 0 ? 'border-info bg-info/10' : 'border-border bg-card',
            )}
          >
            <span className="rounded-md bg-muted px-2.5 py-1 font-mono text-[13px] font-bold">
              v{entry.version}
            </span>
            <div className="flex min-w-0 flex-1 flex-col gap-0.5">
              <span className="truncate font-semibold">
                {entry.message}
                {index === 0 ? ' · current' : ''}
              </span>
              <span className="text-[13px] text-muted-foreground">
                {entry.author} · {timeAgo(entry.at)}
              </span>
            </div>
            <span className="flex items-center gap-1.5 font-mono text-xs text-info">
              <GithubMark className="size-3" />
              {entry.commit.slice(0, SHORT_COMMIT)}
            </span>
            {index > 0 ? (
              <Button
                variant="secondary"
                size="sm"
                disabled={restore.isPending}
                onClick={() =>
                  restore.mutate(
                    { secretId, commit: entry.commit },
                    {
                      onSuccess: () => toast.success(`Restored v${entry.version} as a new version`),
                    },
                  )
                }
              >
                Restore
              </Button>
            ) : null}
          </li>
        ))}
      </ol>
    </div>
  )
}
