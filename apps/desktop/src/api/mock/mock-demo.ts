import { newId, now, state, type MockSecret, type MockVault } from './mock-state'

const DAY = 86_400

function secret(
  partial: Partial<MockSecret> & Pick<MockSecret, 'name' | 'value'> & { age?: number },
): MockSecret {
  const { age = 3, ...rest } = partial
  const at = now() - age * DAY
  return {
    id: newId(),
    kind: 'api_key',
    environment: null,
    tags: [],
    notes: '',
    favorite: false,
    expires_at: null,
    remind_before_days: null,
    created_at: at,
    updated_at: at,
    version: 1,
    versions: [],
    ...rest,
  }
}

/** Seeds a signed-in demo account with one locked vault (password `demo-password-1`). */
export function seedDemo(login = 'octocat'): void {
  state.ghAccounts = ['octocat', 'hubot']
  state.account = { login, id: login === 'octocat' ? 1 : 2, name: login, avatar_url: '' }
  const vault: MockVault = {
    id: newId(),
    name: 'Personal vault',
    description: 'Demo data',
    repo: 'octocat/gitvault-personal',
    password: 'demo-password-1',
    recoveryKey: 'DEMO-DEMO-DEMO-DEMO-DEMO-DEMO-DEMO-DEMO',
    unlocked: false,
    secrets: [
      secret({
        name: 'STRIPE_SECRET_KEY',
        value: 'sk_live_51Hx2bQ8kLm0PzR7vW3nT9aQz',
        environment: 'Production',
        tags: ['billing'],
        favorite: true,
        updated_at: 3,
      }),
      secret({
        name: 'DATABASE_URL',
        value: 'postgres://app:pw@db.internal/app',
        environment: 'Production',
        updated_at: 7,
      }),
      secret({
        name: 'AWS_ACCESS_KEY_ID',
        value: 'AKIAIOSFODNN7EXAMPLE',
        environment: 'Production',
        updated_at: 14,
      }),
      secret({
        name: 'SENDGRID_API_KEY',
        value: 'SG.abcdefghijklmnop',
        environment: 'Staging',
        updated_at: 30,
      }),
      secret({
        name: 'GITHUB_TOKEN',
        value: 'ghp_exampleexampleexample',
        environment: 'Development',
        expires_at: now() + 12 * DAY,
        updated_at: 2,
      }),
    ],
  }
  state.vaults = [vault]
  state.current = vault.id
}
