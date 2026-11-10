import type { CSSProperties, ReactNode } from 'react'

import { TitleBar } from '@/components/general/title-bar'
import { UpdateNotifier } from '@/components/general/update-notifier'
import { useWindowControls } from '@/hooks/use-window'

const TITLEBAR_HEIGHT = '36px'

/** Wraps every route: reserves room for the custom title bar when running as the desktop app. */
export function RootFrame({ children }: { children: ReactNode }) {
  const { available } = useWindowControls()
  const style = { '--titlebar-h': available ? TITLEBAR_HEIGHT : '0px' } as CSSProperties
  return (
    <div className="pt-(--titlebar-h)" style={style}>
      <TitleBar />
      <UpdateNotifier />
      {children}
    </div>
  )
}
