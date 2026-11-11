import type { AccountDto, AuditEvent, Settings, SecretKind, SecretSummary } from '@/types'

/** In-memory data used by the browser-only mock backend (never shipped in the desktop app path). */
export interface MockVersion {
  commit: string
  message: string
  at: number
  snapshot: Omit<MockSecret, 'versions'>
}

export interface MockSecret {
  id: string
  name: string
  kind: SecretKind
  environment: string | null
  tags: string[]
  value: string
  notes: string
  favorite: boolean
  expires_at: number | null
  remind_before_days: number | null
  created_at: number
  updated_at: number
  version: number
  versions: MockVersion[]
}

export interface MockVault {
  id: string
  name: string
  description: string
  repo: string | null
  password: string
  recoveryKey: string
  unlocked: boolean
  secrets: MockSecret[]
}

export interface MockState {
  account: AccountDto | null
  /** Logins the mock `gh` CLI knows. Empty = signed out of gh. */
  ghAccounts: string[]
  vaults: MockVault[]
  current: string | null
  settings: Settings
  audit: AuditEvent[]
  polls: number
  lastSync: number | null
}

export const state: MockState = {
  account: null,
  ghAccounts: [],
  vaults: [],
  current: null,
  settings: {
    auto_lock_minutes: 5,
    clear_clipboard_seconds: 30,
    hide_revealed_seconds: 30,
    theme: 'dark',
    auto_sync: true,
    github_client_id: null,
  },
  audit: [],
  polls: 0,
  lastSync: null,
}

export const now = () => Math.floor(Date.now() / 1000)
export const newId = () => crypto.randomUUID()

export function toSummary(secret: MockSecret): SecretSummary {
  const days = secret.expires_at === null ? null : Math.floor((secret.expires_at - now()) / 86_400)
  return {
    id: secret.id,
    name: secret.name,
    kind: secret.kind,
    environment: secret.environment,
    tags: secret.tags,
    favorite: secret.favorite,
    expires_at: secret.expires_at,
    expires_in_days: days,
    created_at: secret.created_at,
    updated_at: secret.updated_at,
    version: secret.version,
  }
}

export function record(
  vaultId: string,
  kind: AuditEvent['kind'],
  name: string | null,
  id: string | null,
) {
  state.audit.unshift({
    at: now(),
    kind,
    vault_id: vaultId,
    secret_id: id,
    secret_name: name,
    device: 'Browser · mock',
    detail: null,
  })
}
