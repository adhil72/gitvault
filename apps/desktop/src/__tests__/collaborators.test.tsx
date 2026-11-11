import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { seedDemo } from '@/api/mock/mock-demo'
import { resetMock } from '@/api/mock/mock-backend'
import { state } from '@/api/mock/mock-state'
import { CollaboratorsCard } from '@/components/pages/settings-page/tabs/collaborators-card'
import { renderPage } from '@/lib/test-utils'

beforeEach(() => resetMock())

async function renderCard() {
  seedDemo()
  const vaultId = state.vaults[0]?.id ?? ''
  await renderPage(() => <CollaboratorsCard vaultId={vaultId} />)
  return vaultId
}

describe('collaborators', () => {
  it('invites a user, shows the pending invitation and cancels it', async () => {
    await renderCard()
    const user = userEvent.setup()

    await user.type(await screen.findByLabelText('GitHub username'), '@octocat-friend')
    await user.click(screen.getByRole('button', { name: 'Invite' }))

    expect(await screen.findByText(/invitation pending/)).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Cancel invite' }))

    await waitFor(() => expect(screen.queryByText('octocat-friend')).not.toBeInTheDocument())
  })

  it('shows nothing for a vault without a GitHub repository', async () => {
    seedDemo()
    const vault = state.vaults[0]
    if (vault) vault.repo = null
    await renderPage(() => <CollaboratorsCard vaultId={vault?.id ?? ''} />)
    expect(screen.queryByText('Collaborators')).not.toBeInTheDocument()
  })
})
