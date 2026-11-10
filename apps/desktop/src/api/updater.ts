import type { Update } from '@tauri-apps/plugin-updater'

import { isTauri } from './client'

export interface AvailableUpdate {
  version: string
  notes: string | null
}

// The plugin's Update handle is needed again to install, so it is kept here.
let pending: Update | null = null

/** Asks the update feed for a newer signed release. Always null in the browser build. */
export async function checkForUpdate(): Promise<AvailableUpdate | null> {
  if (!isTauri()) return null
  const { check } = await import('@tauri-apps/plugin-updater')
  pending = await check()
  return pending ? { version: pending.version, notes: pending.body ?? null } : null
}

/** Downloads and installs the update found by `checkForUpdate`, then restarts the app. */
export async function installUpdate(): Promise<void> {
  if (!pending) return
  await pending.downloadAndInstall()
  const { relaunch } = await import('@tauri-apps/plugin-process')
  await relaunch()
}

/** The running app version (empty in the browser build). */
export async function appVersion(): Promise<string> {
  if (!isTauri()) return ''
  const { getVersion } = await import('@tauri-apps/api/app')
  return getVersion()
}
