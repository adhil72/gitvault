use gitvault_core::{
    domain::{audit::AuditEvent, settings::Settings},
    dto::{AuditQuery, GenerateInput, GeneratedDto},
};

use super::command;

command!(generate(input: GenerateInput) -> GeneratedDto => |s| s.tools.generate(&input));
command!(audit_list(query: AuditQuery) -> Vec<AuditEvent> => |s| s.audit.list(&query));
command!(settings_get() -> Settings => |s| Ok(s.settings.get()));
command!(settings_set(settings: Settings) -> Settings => |s| s.settings.update(&settings));
