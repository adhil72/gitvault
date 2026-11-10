import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { seedDemo } from '@/api/mock/mock-demo'
import { renderApp, resetMock } from '@/lib/test-utils'
import { state } from '@/api/mock/mock-state'

beforeEach(() => resetMock())

describe('signed-out flows', () => {
  it('sends a signed-out visitor from / to the sign-in page', async () => {
    await renderApp('/')
    expect(await screen.findByRole('heading', { name: 'Sign in to GitVault' })).toBeInTheDocument()
  })

  it('protects vault routes until the user signs in', async () => {
    await renderApp('/vaults/anything')
    expect(await screen.findByRole('heading', { name: 'Sign in to GitVault' })).toBeInTheDocument()
  })
})

describe('signed-in vault flows', () => {
  it('opens the current vault and lists its secrets', async () => {
    seedDemo()
    await renderApp('/')
    expect(await screen.findByText('STRIPE_SECRET_KEY')).toBeInTheDocument()
    expect(screen.getByText('DATABASE_URL')).toBeInTheDocument()
  })

  it('shows details, reveals and hides a value, and keeps it masked by default', async () => {
    seedDemo()
    const user = userEvent.setup()
    await renderApp('/')
    await user.click(await screen.findByRole('button', { name: /^STRIPE_SECRET_KEY/ }))
    const panel = await screen.findByRole('complementary', { name: 'Secret details' })
    expect(await within(panel).findByTestId('secret-value')).toHaveTextContent('••••')
    await user.click(within(panel).getByRole('button', { name: 'Reveal value' }))
    await waitFor(() =>
      expect(within(panel).getByTestId('secret-value')).toHaveTextContent('sk_live_'),
    )
    await user.click(within(panel).getByRole('button', { name: 'Hide value' }))
    expect(within(panel).getByTestId('secret-value')).toHaveTextContent('••••')
  })

  it('sends the user to the unlock page when the vault is locked', async () => {
    seedDemo()
    const vault = state.vaults[0]
    if (vault) vault.unlocked = false
    await renderApp(`/vaults/${vault?.id}`)
    expect(await screen.findByRole('heading', { name: 'Unlock vault' })).toBeInTheDocument()
  })

  it('unlocks with the master password and rejects a wrong one', async () => {
    seedDemo()
    const vault = state.vaults[0]
    if (vault) vault.unlocked = false
    const user = userEvent.setup()
    await renderApp(`/vaults/${vault?.id}/unlock`)
    const field = await screen.findByLabelText('Master password')
    await user.type(field, 'not the password')
    await user.click(screen.getByRole('button', { name: 'Unlock vault' }))
    expect(await screen.findByText(/wrong master password/i)).toBeInTheDocument()
    await user.clear(field)
    await user.type(field, 'demo-password-1')
    await user.click(screen.getByRole('button', { name: 'Unlock vault' }))
    expect(await screen.findByText('STRIPE_SECRET_KEY')).toBeInTheDocument()
  })
})
