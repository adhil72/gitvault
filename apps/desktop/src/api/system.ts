import { isTauri } from './client'

/** Copies text to the clipboard (Tauri plugin in the app, browser API otherwise). */
export async function writeClipboard(text: string): Promise<void> {
  if (isTauri()) {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
    await writeText(text)
    return
  }
  await navigator.clipboard.writeText(text)
}

/** Clears the clipboard. */
export async function clearClipboard(): Promise<void> {
  if (isTauri()) {
    const { clear } = await import('@tauri-apps/plugin-clipboard-manager')
    await clear()
    return
  }
  await navigator.clipboard.writeText('')
}

/** Opens a URL in the user's browser. */
export async function openExternal(url: string): Promise<void> {
  if (isTauri()) {
    const { openUrl } = await import('@tauri-apps/plugin-opener')
    await openUrl(url)
    return
  }
  window.open(url, '_blank', 'noopener,noreferrer')
}
