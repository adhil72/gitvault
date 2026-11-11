import type {
  AddSecretInput,
  CollaboratorDto,
  CreateVaultInput,
  GenerateInput,
  ImportEnvInput,
  ListSecretsQuery,
  UnlockInput,
  UpdateSecretInput,
} from '@/types'

import { ApiError } from '@/lib/api-error'
import { seedDemo } from './mock-demo'
import { newId, now, record, state, toSummary, type MockSecret, type MockVault } from './mock-state'

type Args = Record<string, unknown>
type Handler = (args: Args) => unknown

const fail = (code: string, message: string): never => {
  throw new ApiError(code, message)
}

function vault(id: unknown): MockVault {
  return state.vaults.find((v) => v.id === id) ?? fail('E_NOT_FOUND', 'vault not found')
}

function open(id: unknown): MockVault {
  const found = vault(id)
  return found.unlocked ? found : fail('E_LOCKED', 'the vault is locked')
}

/** What a locked vault still shows: names and metadata from the local index, never values or notes. */
function browse(id: unknown): MockVault {
  return vault(id)
}

function find(v: MockVault, id: unknown): MockSecret {
  return v.secrets.find((s) => s.id === id) ?? fail('E_NOT_FOUND', 'secret not found')
}

function snapshot(s: MockSecret, message: string) {
  const { versions: _ignored, ...rest } = s
  void _ignored
  s.versions.unshift({
    commit: newId().replaceAll('-', ''),
    message,
    at: now(),
    snapshot: { ...rest },
  })
}

function matches(s: MockSecret, q: ListSecretsQuery): boolean {
  const text = (q.search ?? '').toLowerCase()
  const hay = [s.name, s.environment, ...s.tags].join(' ').toLowerCase()
  const same = (a: string | null, b: string | null) => !b || a?.toLowerCase() === b.toLowerCase()
  return (
    hay.includes(text) &&
    same(s.environment, q.environment) &&
    (!q.tag || s.tags.includes(q.tag)) &&
    (!q.favorites_only || s.favorite) &&
    (!q.kind || s.kind === q.kind)
  )
}

function counts(values: (string | null)[]) {
  const map = new Map<string, number>()
  values.forEach((v) => v && map.set(v, (map.get(v) ?? 0) + 1))
  return [...map].sort(([a], [b]) => a.localeCompare(b)).map(([name, count]) => ({ name, count }))
}

function detail(s: MockSecret, locked = false) {
  return {
    ...toSummary(s),
    notes: locked ? '' : s.notes,
    remind_before_days: s.remind_before_days,
  }
}

function addSecret(i: AddSecretInput) {
  const v = open(i.vault_id)
  if (!i.value) fail('E_VALIDATION', 'a secret needs a value')
  if (
    v.secrets.some(
      (s) =>
        s.name.toLowerCase() === i.name.toLowerCase() && s.environment === (i.environment || null),
    )
  )
    fail('E_CONFLICT', `a secret named ${i.name} already exists in that environment`)
  const at = now()
  const created: MockSecret = {
    id: newId(),
    name: i.name.trim(),
    kind: i.kind,
    environment: i.environment || null,
    tags: i.tags,
    value: i.value,
    notes: i.notes ?? '',
    favorite: false,
    expires_at: i.expires_at,
    remind_before_days: i.remind_before_days,
    created_at: at,
    updated_at: at,
    version: 1,
    versions: [],
  }
  snapshot(created, `secret: add ${created.id.slice(0, 8)}`)
  v.secrets.push(created)
  record(v.id, 'created', created.name, created.id)
  return detail(created)
}

function updateSecret(i: UpdateSecretInput) {
  const s = find(open(i.vault_id), i.secret_id)
  Object.assign(s, {
    name: i.name,
    kind: i.kind,
    environment: i.environment || null,
    tags: i.tags,
    notes: i.notes ?? '',
    expires_at: i.expires_at,
    remind_before_days: i.remind_before_days,
    favorite: i.favorite,
    updated_at: now(),
    version: s.version + 1,
  })
  if (i.value) s.value = i.value
  snapshot(s, `secret: update ${s.id.slice(0, 8)}`)
  record(i.vault_id, 'updated', s.name, s.id)
  return detail(s)
}

function createVault(i: CreateVaultInput) {
  if (i.master_password.length < 8)
    fail('E_VALIDATION', 'master password must be at least 8 characters')
  const v: MockVault = {
    id: newId(),
    name: i.name.trim(),
    description: i.description ?? '',
    repo: i.create_remote
      ? `${state.account?.login ?? 'you'}/${i.repo_name || 'gitvault-' + i.name.toLowerCase().replaceAll(' ', '-')}`
      : null,
    password: i.master_password,
    recoveryKey: 'ABCD-EFGH-IJKL-MNOP-QRST-UVWX-YZ23-4567',
    unlocked: true,
    secrets: [],
  }
  state.vaults.push(v)
  state.current = v.id
  return { vault: summaryOf(v), recovery_key: v.recoveryKey, remote_error: null }
}

function summaryOf(v: MockVault) {
  return {
    id: v.id,
    name: v.name,
    description: v.description,
    repo: v.repo,
    state: 'local' as const,
    unlocked: v.unlocked,
    current: state.current === v.id,
  }
}

function unlock(i: UnlockInput) {
  const v = vault(i.vault_id)
  const ok = i.password ? i.password === v.password : i.recovery_key === v.recoveryKey
  if (!ok) fail('E_WRONG_PASSWORD', 'wrong master password or recovery key')
  v.unlocked = true
  state.current = v.id
  record(v.id, 'unlocked', null, null)
  return summaryOf(v)
}

function generate(i: GenerateInput) {
  const pool =
    i.kind === 'pin'
      ? '0123456789'
      : 'abcdefghijkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789!@#$%'
  const length = i.kind === 'passphrase' ? i.words * 5 : i.length
  const value = Array.from({ length }, () => pool[Math.floor(Math.random() * pool.length)]).join('')
  const bits = Math.round(length * Math.log2(pool.length))
  return {
    value,
    entropy_bits: bits,
    strength: bits < 40 ? 'weak' : bits < 60 ? 'fair' : bits < 80 ? 'strong' : 'very strong',
  }
}

function importEnv(i: ImportEnvInput) {
  const v = open(i.vault_id)
  let created = 0
  for (const line of i.content.split('\n')) {
    const [name, ...rest] = line.split('=')
    if (!name || line.startsWith('#') || !rest.length) continue
    addSecret({
      vault_id: v.id,
      name: name.trim(),
      kind: 'api_key',
      value: rest.join('=').replace(/^"|"$/g, ''),
      environment: i.environment,
      tags: [],
      notes: null,
      expires_at: null,
      remind_before_days: null,
    })
    created += 1
  }
  return { created, updated: 0, skipped: 0 }
}

/** Collaborators per vault id, kept outside `state` because the mock has no GitHub behind it. */
const collaborators = new Map<string, CollaboratorDto[]>()

const peopleOf = (vaultId: unknown): CollaboratorDto[] => collaborators.get(String(vaultId)) ?? []

const handlers: Record<string, Handler> = {
  app_info: () => ({ name: 'GitVault', version: 'mock' }),
  auth_status: () => state.account,
  auth_login_start: () => ({
    device_code: 'mock-device',
    user_code: 'WDJB-MJHT',
    verification_uri: 'https://github.com/login/device',
    expires_in: 900,
    interval: 1,
  }),
  auth_login_poll: () => {
    state.polls += 1
    if (state.polls < 3) return { status: 'pending' }
    seedDemo()
    return { status: 'authorized', account: state.account }
  },
  auth_login_token: (a) => {
    if (!a.token) fail('E_NOT_AUTHENTICATED', 'not signed in to GitHub')
    if (a.token === 'demo') seedDemo()
    else state.account = { login: 'octocat', id: 1, name: null, avatar_url: '' }
    return state.account
  },
  auth_login_gh: () => {
    seedDemo()
    return state.account
  },
  auth_gh_accounts: () =>
    state.ghAccounts.map((login) => ({ login, current: state.account?.login === login })),
  auth_login_gh_account: (a) => {
    if (!state.ghAccounts.includes(String(a.login))) fail('E_VALIDATION', 'unknown gh account')
    seedDemo(String(a.login))
    return state.account
  },
  auth_logout: () => {
    state.account = null
  },
  vault_unlock_trusted: () => fail('E_LOCKED', 'this device is not trusted yet'),
  device_list: () => [],
  device_trust: (a) => ({
    device: { id: 'mock-device', name: String(a.name), created_at: 0, this_device: true },
    device_key: null,
  }),
  device_revoke: () => undefined,
  collaborator_list: (a) => peopleOf(a.vaultId),
  collaborator_invite: (a) => {
    const input = a.input as { vault_id: string; login: string; role: CollaboratorDto['role'] }
    const login = input.login.replace(/^@/, '')
    const rest = peopleOf(input.vault_id).filter((p) => p.login !== login)
    collaborators.set(input.vault_id, [
      ...rest,
      { login, avatar_url: '', role: input.role, pending: true },
    ])
    return undefined
  },
  collaborator_remove: (a) => {
    collaborators.set(
      String(a.vaultId),
      peopleOf(a.vaultId).filter((p) => p.login !== a.login),
    )
    return undefined
  },
  vault_discover: () => ({ vaults: [], remote_error: null }),
  vault_list: () => ({ vaults: state.vaults.map(summaryOf), remote_error: null }),
  vault_create: (a) => createVault(a.input as CreateVaultInput),
  vault_connect: () => fail('E_NOT_FOUND', 'repository not found'),
  vault_unlock: (a) => unlock(a.input as UnlockInput),
  vault_lock: (a) => {
    vault(a.vaultId).unlocked = false
  },
  vault_info: (a) => summaryOf(vault(a.vaultId)),
  vault_use: (a) => {
    state.current = vault(a.vaultId).id
    return summaryOf(vault(a.vaultId))
  },
  vault_forget: (a) => {
    state.vaults = state.vaults.filter((v) => v.id !== a.vaultId)
  },
  vault_current: () => state.current,
  vault_change_password: (a) => {
    const v = vault(a.vaultId)
    if (v.password !== a.current) fail('E_WRONG_PASSWORD', 'wrong master password or recovery key')
    v.password = String(a.newPassword)
  },
  vault_regenerate_recovery: (a) =>
    vault(a.vaultId).password === a.current
      ? vault(a.vaultId).recoveryKey
      : fail('E_WRONG_PASSWORD', 'wrong master password'),
  secret_list: (a) => {
    const q = a.query as ListSecretsQuery
    const items = browse(q.vault_id)
      .secrets.filter((s) => matches(s, q))
      .map(toSummary)
    return items.sort((x, y) =>
      q.sort === 'name' ? x.name.localeCompare(y.name) : y.updated_at - x.updated_at,
    )
  },
  secret_facets: (a) => {
    const all = browse(a.vaultId).secrets
    return {
      total: all.length,
      favorites: all.filter((s) => s.favorite).length,
      expiring_soon: all.filter((s) => s.expires_at !== null && s.expires_at - now() < 30 * 86_400)
        .length,
      environments: counts(all.map((s) => s.environment)),
      tags: counts(all.flatMap((s) => s.tags)),
    }
  },
  secret_get: (a) => detail(find(browse(a.vaultId), a.secretId), !vault(a.vaultId).unlocked),
  secret_reveal: (a) => {
    const s = find(open(a.vaultId), a.secretId)
    record(String(a.vaultId), 'viewed', s.name, s.id)
    return { value: s.value }
  },
  secret_mark_copied: (a) => {
    const s = find(open(a.vaultId), a.secretId)
    record(String(a.vaultId), 'copied', s.name, s.id)
  },
  secret_add: (a) => addSecret(a.input as AddSecretInput),
  secret_update: (a) => updateSecret(a.input as UpdateSecretInput),
  secret_rotate: (a) => {
    const s = find(open(a.vaultId), a.secretId)
    Object.assign(s, { value: String(a.value), version: s.version + 1, updated_at: now() })
    snapshot(s, `secret: rotate ${s.id.slice(0, 8)}`)
    record(String(a.vaultId), 'rotated', s.name, s.id)
    return detail(s)
  },
  secret_delete: (a) => {
    const v = open(a.vaultId)
    const s = find(v, a.secretId)
    v.secrets = v.secrets.filter((x) => x.id !== s.id)
    record(v.id, 'deleted', s.name, s.id)
  },
  secret_history: (a) =>
    find(open(a.vaultId), a.secretId).versions.map((v, i, all) => ({
      commit: v.commit,
      version: all.length - i,
      message: v.message,
      author: 'octocat',
      at: v.at,
    })),
  secret_restore: (a) => {
    const s = find(open(a.vaultId), a.secretId)
    const old =
      s.versions.find((v) => v.commit === a.commit) ?? fail('E_NOT_FOUND', 'version not found')
    Object.assign(s, { ...old.snapshot, version: s.version + 1, updated_at: now() })
    return detail(s)
  },
  import_preview: (a) =>
    (a.input as ImportEnvInput).content
      .split('\n')
      .filter((l) => l.includes('=') && !l.startsWith('#'))
      .map((l) => ({
        name: l.split('=')[0]?.trim() ?? '',
        masked_value: '••••••••',
        status: 'new',
      })),
  import_env: (a) => importEnv(a.input as ImportEnvInput),
  export_env: (a) =>
    open(a.vaultId)
      .secrets.map((s) => `${s.name}=${s.value}`)
      .join('\n') + '\n',
  sync_status: () => ({
    state: state.lastSync ? 'synced' : 'pending',
    pending: state.lastSync ? 0 : 1,
    conflicts: 0,
    last_sync: state.lastSync,
  }),
  sync_now: () => {
    state.lastSync = now()
    return { pulled: 0, pushed: 1, conflicts: 0, no_remote: false }
  },
  sync_conflicts: () => [],
  sync_resolve: () => undefined,
  generate: (a) => generate(a.input as GenerateInput),
  audit_list: () => state.audit,
  settings_get: () => state.settings,
  settings_set: (a) => {
    Object.assign(state.settings, a.settings)
    return state.settings
  },
}

/** Handles a command in the browser (development and tests only). */
export async function mockInvoke(command: string, args: Args): Promise<unknown> {
  await new Promise((resolve) => setTimeout(resolve, 40))
  const handler =
    handlers[command] ?? (() => fail('E_INTERNAL', `mock: unknown command ${command}`))
  return handler(args)
}

/** Resets the mock to a clean, signed-out state (used by tests). */
export function resetMock(): void {
  collaborators.clear()
  Object.assign(state, {
    account: null,
    ghAccounts: [],
    vaults: [],
    current: null,
    audit: [],
    polls: 0,
    lastSync: null,
  })
}
