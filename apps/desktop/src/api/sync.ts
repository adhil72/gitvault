import type { ConflictDto, ConflictKeep, SyncResult, SyncStatus } from '@/types'

import { call } from './client'

export const syncStatus = (vaultId: string) => call<SyncStatus>('sync_status', { vaultId })
export const syncNow = (vaultId: string) => call<SyncResult>('sync_now', { vaultId })
export const syncConflicts = (vaultId: string) => call<ConflictDto[]>('sync_conflicts', { vaultId })
export const syncResolve = (vaultId: string, secretId: string, keep: ConflictKeep) =>
  call<void>('sync_resolve', { vaultId, secretId, keep })
