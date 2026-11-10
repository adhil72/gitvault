import { useStartupUpdateNotice } from '@/hooks/use-updater'

/** Mounted once; offers an update toast shortly after launch. */
export function UpdateNotifier() {
  useStartupUpdateNotice()
  return null
}
