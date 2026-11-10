export interface AuthPanelContent {
  headline: string
  description: string
}

const DEFAULT_AUTH_PANEL: AuthPanelContent = {
  headline: 'Your secrets, signed in with GitHub',
  description:
    'Authorize once with your GitHub account. Everything else stays encrypted on this computer.',
}

const AUTH_PANEL_BY_PATH: Record<string, AuthPanelContent> = {
  '/sign-in': DEFAULT_AUTH_PANEL,
  '/sign-in/device': {
    headline: 'Authorize this device',
    description: 'Finish signing in on GitHub, then come back. GitVault continues automatically.',
  },
}

/** Headline and description shown in the brand panel for an auth route. */
export function getAuthPanelContent(pathname: string): AuthPanelContent {
  return AUTH_PANEL_BY_PATH[pathname] ?? DEFAULT_AUTH_PANEL
}
