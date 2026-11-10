import { PageHeading } from '@/components/general/page-heading'

import { GhCliSignIn } from './gh-cli-sign-in'
import { PermissionList } from './permission-list'

export function SignInPage() {
  return (
    <div className="flex w-[360px] flex-col gap-6">
      <PageHeading
        title="Sign in to GitVault"
        description="Uses your GitHub CLI session to create or unlock your vault."
      />
      <GhCliSignIn />
      <PermissionList />
      <p className="text-xs text-muted-foreground">
        By continuing you agree to the Terms and Privacy Policy.
      </p>
    </div>
  )
}
