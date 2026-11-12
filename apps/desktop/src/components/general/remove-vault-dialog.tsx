import { useNavigate } from '@tanstack/react-router'
import { toast } from 'sonner'

import { ConfirmDialog } from '@/components/general/confirm-dialog'
import { useForgetVault, useRemoteStatus, useVault } from '@/hooks/use-vaults'

interface RemoveVaultDialogProps {
  open: boolean
  vaultId: string
  onClose: () => void
}

/** Removes the local copy of a vault. It never needs the master password and never touches GitHub. */
export function RemoveVaultDialog({ open, vaultId, onClose }: RemoveVaultDialogProps) {
  const navigate = useNavigate()
  const forget = useForgetVault()
  const vaultName = useVault(vaultId).data?.name ?? 'this vault'
  // With the repository gone this device may hold the only copy, so the warning is stronger.
  const remoteMissing = useRemoteStatus(vaultId).data === 'missing'

  const confirm = () =>
    forget.mutate(vaultId, {
      onSuccess: () => {
        onClose()
        void navigate({ to: '/vaults' })
      },
      onError: (error) => toast.error(error.message),
    })

  return (
    <ConfirmDialog
      open={open}
      destructive
      title={`Remove ${vaultName} from this device?`}
      description={
        remoteMissing
          ? 'The GitHub repository was deleted, so this device may hold the only copy. Once removed, the secrets cannot be recovered.'
          : 'Only the copy on this device is removed. The GitHub repository is not touched; to delete it, do it on GitHub.'
      }
      confirmLabel="Remove from this device"
      busy={forget.isPending}
      onConfirm={confirm}
      onCancel={onClose}
    />
  )
}
