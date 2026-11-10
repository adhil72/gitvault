import { openExternal } from '@/api/system'

/** Opens a URL in the user's browser (Tauri opener plugin in the desktop app). */
export const useOpenExternal = () => openExternal
