export interface GithubPermission {
  label: string
  scope: string
}

/**
 * Scopes requested at sign-in. `repo` is required to create a private vault repository
 * and push encrypted commits to it. Keep in sync with the device flow in `gitvault-infra`.
 */
export const GITHUB_PERMISSIONS: readonly GithubPermission[] = [
  { label: 'Read your public profile', scope: 'read:user' },
  { label: 'Create and sync private vault repositories', scope: 'repo' },
]
