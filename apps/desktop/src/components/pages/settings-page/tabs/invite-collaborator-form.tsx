import { useState, type FormEvent } from 'react'
import { toast } from 'sonner'

import { ErrorNotice } from '@/components/general/error-notice'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { useInviteCollaborator } from '@/hooks/use-collaborators'

/** Invites a GitHub user to the vault's repository, read-only unless the switch is on. */
export function InviteCollaboratorForm({ vaultId }: { vaultId: string }) {
  const [login, setLogin] = useState('')
  const [canWrite, setCanWrite] = useState(false)
  const invite = useInviteCollaborator(vaultId)
  const name = login.trim().replace(/^@/, '')

  const submit = (event: FormEvent) => {
    event.preventDefault()
    invite.mutate(
      { login: name, role: canWrite ? 'write' : 'read' },
      {
        onSuccess: () => {
          toast.success(`Invited ${name}`)
          setLogin('')
        },
      },
    )
  }

  return (
    <form onSubmit={submit} className="flex flex-col gap-4 px-5 py-4">
      <ErrorNotice error={invite.error} />
      <div className="flex flex-col gap-2">
        <Label htmlFor="collaborator-login">GitHub username</Label>
        <Input
          id="collaborator-login"
          value={login}
          onChange={(event) => setLogin(event.target.value)}
          placeholder="octocat"
          autoComplete="off"
          spellCheck={false}
        />
      </div>
      <div className="flex items-center justify-between gap-6">
        <div className="flex flex-col gap-0.5">
          <Label htmlFor="collaborator-write">Can change secrets</Label>
          <span className="text-[13px] leading-snug text-muted-foreground">
            Off means read-only: they can open the vault but not push changes.
          </span>
        </div>
        <Switch id="collaborator-write" checked={canWrite} onCheckedChange={setCanWrite} />
      </div>
      <p className="text-[13px] leading-snug text-muted-foreground">
        Inviting grants repository access only. Share the master password with them separately.
      </p>
      <div className="flex justify-end">
        <Button type="submit" disabled={!name || invite.isPending}>
          {invite.isPending ? 'Inviting…' : 'Invite'}
        </Button>
      </div>
    </form>
  )
}
