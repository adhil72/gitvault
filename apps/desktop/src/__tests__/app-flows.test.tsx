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

const PASSWORD = 'demo-password-1'

/** Seeds the demo account, opens its vault and (optionally) unlocks it first. */
function seedVault(unlocked: boolean) {
  seedDemo()
  const vault = state.vaults[0]
  if (vault) vault.unlocked = unlocked
  return vault?.id ?? ''
}

describe('opening the app', () => {
  it('asks which vault to open instead of jumping into one', async () => {
    seedVault(false)
    await renderApp('/')
    expect(await screen.findByRole('heading', { name: 'Your vaults' })).toBeInTheDocument()
    expect(await screen.findByText('Personal vault')).toBeInTheDocument()
  })

  it('signs in silently with the gh CLI account when one exists', async () => {
    seedVault(false)
    state.account = null
    await renderApp('/')
    expect(await screen.findByRole('heading', { name: 'Your vaults' })).toBeInTheDocument()
  })

  it('switches between gh accounts from the header', async () => {
    const user = userEvent.setup()
    await renderApp(`/vaults/${seedVault(false)}`)
    await user.click(await screen.findByRole('button', { name: 'Account menu' }))
    await user.click(await screen.findByRole('menuitem', { name: 'hubot' }))
    expect(await screen.findByRole('heading', { name: 'Your vaults' })).toBeInTheDocument()
    expect(state.account?.login).toBe('hubot')
  })
})

describe('locked vault', () => {
  it('opens without a password and lists names, with values locked', async () => {
    await renderApp(`/vaults/${seedVault(false)}`)
    expect(await screen.findByText('STRIPE_SECRET_KEY')).toBeInTheDocument()
    expect(screen.getByText(/this vault is locked/i)).toBeInTheDocument()
  })

  it('asks for the password in a dialog before revealing, then reveals', async () => {
    const user = userEvent.setup()
    await renderApp(`/vaults/${seedVault(false)}`)
    await user.click(await screen.findByRole('button', { name: /^STRIPE_SECRET_KEY/ }))
    const panel = await screen.findByRole('complementary', { name: 'Secret details' })
    await user.click(await within(panel).findByRole('button', { name: 'Reveal value' }))

    const dialog = await screen.findByRole('dialog')
    await user.type(within(dialog).getByLabelText('Master password'), 'not the password')
    await user.click(within(dialog).getByRole('button', { name: 'Unlock vault' }))
    expect(await within(dialog).findByText(/wrong master password/i)).toBeInTheDocument()

    await user.clear(within(dialog).getByLabelText('Master password'))
    await user.type(within(dialog).getByLabelText('Master password'), PASSWORD)
    await user.click(within(dialog).getByRole('button', { name: 'Unlock vault' }))
    await waitFor(() =>
      expect(within(panel).getByTestId('secret-value')).toHaveTextContent('sk_live_'),
    )
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })
})

describe('unlocked vault', () => {
  it('shows details, reveals and hides a value, and keeps it masked by default', async () => {
    const user = userEvent.setup()
    await renderApp(`/vaults/${seedVault(true)}`)
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
})
