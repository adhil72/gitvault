use gitvault_core::dto::{AccountDto, DeviceCodeDto, GhAccountDto, LoginPollDto};

use super::command;

command!(auth_status() -> Option<AccountDto> => |s| Ok(s.auth.status()));
command!(auth_login_start() -> DeviceCodeDto => |s| s.auth.login_start());
command!(auth_login_poll(device_code: String) -> LoginPollDto => |s| s.auth.login_poll(&device_code));
command!(auth_login_token(token: String) -> AccountDto => |s| s.auth.login_with_token(&token));
command!(auth_login_gh() -> AccountDto => |s| s.auth.login_with_gh_cli());
command!(auth_gh_accounts() -> Vec<GhAccountDto> => |s| Ok(s.auth.gh_accounts()));
command!(auth_login_gh_account(login: String) -> AccountDto => |s| s.auth.login_with_gh_account(&login));
command!(auth_logout() -> () => |s| s.auth.logout());
