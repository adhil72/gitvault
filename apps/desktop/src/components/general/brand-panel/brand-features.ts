import { Lock, Terminal, type LucideIcon } from 'lucide-react'

import { GithubMark } from '../github-mark'

export interface BrandFeature {
  id: string
  title: string
  description: string
  icon: LucideIcon | typeof GithubMark
}

export const BRAND_FEATURES: readonly BrandFeature[] = [
  {
    id: 'github',
    title: 'Sign in with GitHub',
    description: 'Your GitHub account identifies your vault. No new password to manage.',
    icon: GithubMark,
  },
  {
    id: 'encrypted',
    title: 'Encrypted on your device',
    description: 'AES-256-GCM with a key derived from your master password.',
    icon: Lock,
  },
  {
    id: 'developers',
    title: 'Built for developers',
    description: 'Inject secrets into .env files and the CLI.',
    icon: Terminal,
  },
]
