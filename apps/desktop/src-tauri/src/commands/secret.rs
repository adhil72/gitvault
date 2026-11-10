use gitvault_core::dto::{
    AddSecretInput, EnvFilter, Facets, HistoryEntry, ImportEnvInput, ImportPreviewEntry,
    ImportReport, ListSecretsQuery, RevealedSecret, SecretDetail, SecretSummary, UpdateSecretInput,
};

use super::command;

command!(secret_list(query: ListSecretsQuery) -> Vec<SecretSummary> => |s| s.secrets.list(&query));
command!(secret_facets(vault_id: String) -> Facets => |s| s.secrets.facets(&vault_id));
command!(secret_get(vault_id: String, secret_id: String) -> SecretDetail => |s| s.secrets.get(&vault_id, &secret_id));
command!(secret_reveal(vault_id: String, secret_id: String) -> RevealedSecret => |s| s.secrets.reveal(&vault_id, &secret_id));
command!(secret_mark_copied(vault_id: String, secret_id: String) -> () => |s| s.secrets.mark_copied(&vault_id, &secret_id));
command!(secret_add(input: AddSecretInput) -> SecretDetail => |s| s.secrets.add(&input));
command!(secret_update(input: UpdateSecretInput) -> SecretDetail => |s| s.secrets.update(&input));
command!(secret_rotate(vault_id: String, secret_id: String, value: String) -> SecretDetail => |s| {
    s.secrets.rotate(&vault_id, &secret_id, &value)
});
command!(secret_delete(vault_id: String, secret_id: String) -> () => |s| s.secrets.delete(&vault_id, &secret_id));
command!(secret_history(vault_id: String, secret_id: String) -> Vec<HistoryEntry> => |s| s.secrets.history(&vault_id, &secret_id));
command!(secret_restore(vault_id: String, secret_id: String, commit: String) -> SecretDetail => |s| {
    s.secrets.restore(&vault_id, &secret_id, &commit)
});
command!(import_preview(input: ImportEnvInput) -> Vec<ImportPreviewEntry> => |s| s.secrets.preview_env(&input));
command!(import_env(input: ImportEnvInput) -> ImportReport => |s| s.secrets.import_env(&input));
command!(export_env(vault_id: String, filter: EnvFilter) -> String => |s| s.secrets.export_env(&vault_id, &filter));
