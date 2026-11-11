import type {
  CollaboratorDto,
  CreatedVault,
  CreateVaultInput,
  DeviceDto,
  InviteCollaboratorInput,
  TrustedDeviceDto,
  UnlockInput,
  VaultList,
  VaultSummary,
} from '@/types'

import { call } from './client'

export const vaultList = () => call<VaultList>('vault_list')
export const vaultDiscover = () => call<VaultList>('vault_discover')
export const vaultCreate = (input: CreateVaultInput) =>
  call<CreatedVault>('vault_create', { input })
export const vaultConnect = (repo: string) => call<VaultSummary>('vault_connect', { repo })
export const vaultUnlock = (input: UnlockInput) => call<VaultSummary>('vault_unlock', { input })
export const vaultUnlockTrusted = (vaultId: string) =>
  call<VaultSummary>('vault_unlock_trusted', { vaultId })
export const deviceTrust = (vaultId: string, name: string) =>
  call<TrustedDeviceDto>('device_trust', { vaultId, name })
export const deviceList = (vaultId: string) => call<DeviceDto[]>('device_list', { vaultId })
export const deviceRevoke = (vaultId: string, deviceId: string) =>
  call<void>('device_revoke', { vaultId, deviceId })
export const collaboratorList = (vaultId: string) =>
  call<CollaboratorDto[]>('collaborator_list', { vaultId })
export const collaboratorInvite = (input: InviteCollaboratorInput) =>
  call<void>('collaborator_invite', { input })
export const collaboratorRemove = (vaultId: string, login: string) =>
  call<void>('collaborator_remove', { vaultId, login })
export const vaultLock = (vaultId: string) => call<void>('vault_lock', { vaultId })
export const vaultInfo = (vaultId: string) => call<VaultSummary>('vault_info', { vaultId })
export const vaultUse = (vaultId: string) => call<VaultSummary>('vault_use', { vaultId })
export const vaultForget = (vaultId: string) => call<void>('vault_forget', { vaultId })
export const vaultCurrent = () => call<string | null>('vault_current')
export const vaultChangePassword = (vaultId: string, current: string, newPassword: string) =>
  call<void>('vault_change_password', { vaultId, current, newPassword })
export const vaultRegenerateRecovery = (vaultId: string, current: string) =>
  call<string>('vault_regenerate_recovery', { vaultId, current })
