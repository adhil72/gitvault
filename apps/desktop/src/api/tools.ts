import type { AuditEvent, AuditQuery, GenerateInput, GeneratedDto, Settings } from '@/types'

import { call } from './client'

export const generate = (input: GenerateInput) => call<GeneratedDto>('generate', { input })
export const auditList = (query: AuditQuery) => call<AuditEvent[]>('audit_list', { query })
export const settingsGet = () => call<Settings>('settings_get')
export const settingsSet = (settings: Settings) => call<Settings>('settings_set', { settings })
