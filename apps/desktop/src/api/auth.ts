import type { AccountDto, GhAccountDto } from '@/types'

import { call } from './client'

export const authStatus = () => call<AccountDto | null>('auth_status')
export const authLoginGh = () => call<AccountDto>('auth_login_gh')
export const authLogout = () => call<void>('auth_logout')
export const authGhAccounts = () => call<GhAccountDto[]>('auth_gh_accounts')
export const authLoginGhAccount = (login: string) =>
  call<AccountDto>('auth_login_gh_account', { login })
