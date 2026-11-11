import { toast } from 'sonner'

import { FormField } from '@/components/general/form-field'
import { SelectField } from '@/components/general/select-field'
import { useGhAccounts, useSwitchAccount } from '@/hooks/use-auth'

/** Picks which GitHub CLI account owns the new vault; hidden when gh knows no accounts. */
export function AccountField() {
  const { data: accounts = [] } = useGhAccounts()
  const switchAccount = useSwitchAccount()
  const current = accounts.find((gh) => gh.current)?.login ?? ''

  if (accounts.length === 0) return null

  // Repositories are listed per account, so the next step follows the switch.
  const choose = (login: string) => {
    if (login === current) return
    switchAccount.mutate(login, { onError: (error) => toast.error(error.message) })
  }

  return (
    <FormField
      label="GitHub account"
      htmlFor="vault-account"
      hint="The vault is created under this account."
    >
      <SelectField
        id="vault-account"
        value={current}
        onChange={choose}
        options={accounts.map((gh) => ({ value: gh.login, label: gh.login }))}
      />
    </FormField>
  )
}
