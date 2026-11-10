import type { AccountDto } from '@/types'

import { call } from './client'

export const authStatus = () => call<AccountDto | null>('auth_status')
export const authLoginGh = () => call<AccountDto>('auth_login_gh')
export const authLogout = () => call<void>('auth_logout')
