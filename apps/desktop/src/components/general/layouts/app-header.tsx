import { Link, useNavigate, useSearch } from '@tanstack/react-router'
import { Plus, Search } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'

import { AccountMenu } from '@/components/general/layouts/account-menu'
import { Logo } from '@/components/general/logo-mark'
import { SyncIndicator } from '@/components/general/sync-indicator'
import { VaultSwitcher } from '@/components/general/vault-switcher'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useRequireUnlock } from '@/hooks/use-require-unlock'
import type { VaultSearch } from '@/schemas/vault-search'

const SEARCH_DEBOUNCE_MS = 200

function useSearchBox(vaultId: string) {
  const navigate = useNavigate()
  const current = useSearch({ strict: false }) as VaultSearch
  const [text, setText] = useState(current.q ?? '')
  useEffect(() => {
    const timer = setTimeout(() => {
      if ((current.q ?? '') === text) return
      void navigate({
        to: '/vaults/$vaultId',
        params: { vaultId },
        search: { ...current, q: text || undefined },
      })
    }, SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [text, current, navigate, vaultId])
  return { text, setText }
}

function useSlashShortcut(input: React.RefObject<HTMLInputElement | null>) {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement
      const typing =
        ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName) || target.isContentEditable
      if (event.key === '/' && !typing) {
        event.preventDefault()
        input.current?.focus()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [input])
}

export function AppHeader({ vaultId }: { vaultId: string }) {
  const input = useRef<HTMLInputElement>(null)
  const { text, setText } = useSearchBox(vaultId)
  useSlashShortcut(input)
  const navigate = useNavigate()
  const requireUnlock = useRequireUnlock(vaultId)
  const goToAddSecret = () =>
    void navigate({ to: '/vaults/$vaultId/secrets/new/$step', params: { vaultId, step: 'type' } })
  return (
    <header className="flex h-16 shrink-0 items-center justify-between gap-4 border-b border-brand-line bg-inset px-4">
      <div className="flex items-center gap-4">
        <Link to="/vaults/$vaultId" params={{ vaultId }}>
          <Logo size="sm" />
        </Link>
        <span className="h-16 w-px bg-border" />
        <VaultSwitcher vaultId={vaultId} />
      </div>
      <div className="flex items-center gap-3">
        <SyncIndicator vaultId={vaultId} />
        <div className="relative w-80">
          <Search className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            ref={input}
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="Search secrets"
            aria-label="Search secrets"
            className="h-10 pr-8 pl-9"
          />
          <kbd className="absolute top-1/2 right-2.5 -translate-y-1/2 rounded border border-border px-1.5 font-mono text-xs text-muted-foreground">
            /
          </kbd>
        </div>
        <Button onClick={() => requireUnlock(goToAddSecret)}>
          <Plus /> Add secret
        </Button>
        <AccountMenu vaultId={vaultId} />
      </div>
    </header>
  )
}
