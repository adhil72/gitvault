import { useTheme } from 'next-themes'
import { useEffect } from 'react'

import { useSettings } from '@/hooks/use-settings'

/** Applies the saved theme setting (dark, light or system) once settings are loaded. */
export function ThemeSync() {
  const { data } = useSettings()
  const { setTheme } = useTheme()
  const theme = data?.theme
  useEffect(() => {
    if (theme) setTheme(theme)
  }, [theme, setTheme])
  return null
}
