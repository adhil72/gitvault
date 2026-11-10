import type {
  AddSecretInput,
  EnvFilter,
  Facets,
  HistoryEntry,
  ImportEnvInput,
  ImportPreviewEntry,
  ImportReport,
  ListSecretsQuery,
  RevealedSecret,
  SecretDetail,
  SecretSummary,
  UpdateSecretInput,
} from '@/types'

import { call } from './client'

export const secretList = (query: ListSecretsQuery) =>
  call<SecretSummary[]>('secret_list', { query })
export const secretFacets = (vaultId: string) => call<Facets>('secret_facets', { vaultId })
export const secretGet = (vaultId: string, secretId: string) =>
  call<SecretDetail>('secret_get', { vaultId, secretId })
export const secretReveal = (vaultId: string, secretId: string) =>
  call<RevealedSecret>('secret_reveal', { vaultId, secretId })
export const secretMarkCopied = (vaultId: string, secretId: string) =>
  call<void>('secret_mark_copied', { vaultId, secretId })
export const secretAdd = (input: AddSecretInput) => call<SecretDetail>('secret_add', { input })
export const secretUpdate = (input: UpdateSecretInput) =>
  call<SecretDetail>('secret_update', { input })
export const secretRotate = (vaultId: string, secretId: string, value: string) =>
  call<SecretDetail>('secret_rotate', { vaultId, secretId, value })
export const secretDelete = (vaultId: string, secretId: string) =>
  call<void>('secret_delete', { vaultId, secretId })
export const secretHistory = (vaultId: string, secretId: string) =>
  call<HistoryEntry[]>('secret_history', { vaultId, secretId })
export const secretRestore = (vaultId: string, secretId: string, commit: string) =>
  call<SecretDetail>('secret_restore', { vaultId, secretId, commit })
export const importPreview = (input: ImportEnvInput) =>
  call<ImportPreviewEntry[]>('import_preview', { input })
export const importEnv = (input: ImportEnvInput) => call<ImportReport>('import_env', { input })
export const exportEnv = (vaultId: string, filter: EnvFilter) =>
  call<string>('export_env', { vaultId, filter })
