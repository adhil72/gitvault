use gitvault_core::dto::{ConflictDto, ConflictKeep, SyncResult, SyncStatus};

use super::command;

command!(sync_status(vault_id: String) -> SyncStatus => |s| s.sync.status(&vault_id));
command!(sync_now(vault_id: String) -> SyncResult => |s| s.sync.sync_now(&vault_id));
command!(sync_conflicts(vault_id: String) -> Vec<ConflictDto> => |s| s.sync.conflicts(&vault_id));
command!(sync_resolve(vault_id: String, secret_id: String, keep: ConflictKeep) -> () => |s| {
    s.sync.resolve(&vault_id, &secret_id, &keep)
});
