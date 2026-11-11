use gitvault_core::dto::{CollaboratorDto, InviteCollaboratorInput};

use super::command;

command!(collaborator_list(vault_id: String) -> Vec<CollaboratorDto> => |s| s.vaults.list_collaborators(&vault_id));
command!(collaborator_invite(input: InviteCollaboratorInput) -> () => |s| s.vaults.invite_collaborator(&input));
command!(collaborator_remove(vault_id: String, login: String) -> () => |s| s.vaults.remove_collaborator(&vault_id, &login));
