import { invoke } from '@tauri-apps/api/core'

import { ApiError } from '@/lib/api-error'
import type { ApiErrorDto } from '@/types'

export { ApiError }

/** True when running inside the Tauri desktop shell (false in a plain browser). */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function isApiErrorDto(value: unknown): value is ApiErrorDto {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    'message' in value &&
    typeof value.code === 'string' &&
    typeof value.message === 'string'
  )
}

function toApiError(error: unknown): ApiError {
  if (error instanceof ApiError) return error
  if (isApiErrorDto(error)) return new ApiError(error.code, error.message, error.hint)
  return new ApiError('E_INTERNAL', error instanceof Error ? error.message : String(error))
}

/**
 * Calls a backend command. Inside Tauri this is `invoke`; in a plain browser it
 * talks to the in-memory mock backend so the UI can be developed and tested.
 * This is the only function that should reach the backend.
 */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    if (isTauri()) return await invoke<T>(command, args)
    const { mockInvoke } = await import('./mock/mock-backend')
    return (await mockInvoke(command, args ?? {})) as T
  } catch (error) {
    throw toApiError(error)
  }
}
