import { screen } from '@testing-library/react'

import { GITHUB_PERMISSIONS } from '@/config/github-permissions'
import { renderPage } from '@/lib/test-utils'

import { SignInPage } from './sign-in-page'

describe('SignInPage', () => {
  it('offers one-click sign-in with the GitHub CLI', async () => {
    await renderPage(SignInPage)
    expect(await screen.findByRole('button', { name: /continue with github cli/i })).toBeEnabled()
  })

  it('lists every requested GitHub permission', async () => {
    await renderPage(SignInPage)
    for (const { label } of GITHUB_PERMISSIONS) {
      expect(await screen.findByText(label)).toBeInTheDocument()
    }
  })
})
