use std::{
    thread,
    time::{Duration, Instant},
};

use anyhow::{bail, Result};
use gitvault_core::dto::{AccountDto, LoginPollDto};

use crate::{
    app::App,
    cli::{AuthCmd, LoginArgs},
    input,
};

pub fn run(app: &App, cmd: &AuthCmd) -> Result<()> {
    match cmd {
        AuthCmd::Login(args) => login(app, args),
        AuthCmd::Logout => {
            app.services.auth.logout()?;
            app.emit(&serde_json::json!({ "signed_in": false }), || {
                "Signed out.".into()
            })
        }
        AuthCmd::Status => status(app),
        AuthCmd::Accounts => accounts(app),
    }
}

fn status(app: &App) -> Result<()> {
    let account = app.services.auth.status();
    app.emit(&account, || match &account {
        Some(a) => format!("Signed in as {}", a.login),
        None => "Not signed in. Run `gitvault auth login`.".into(),
    })
}

fn accounts(app: &App) -> Result<()> {
    let accounts = app.services.auth.gh_accounts();
    app.emit(&accounts, || {
        if accounts.is_empty() {
            return "No GitHub CLI accounts. Run `gh auth login`.".into();
        }
        let line = |a: &gitvault_core::dto::GhAccountDto| {
            format!("{} {}", if a.current { "*" } else { " " }, a.login)
        };
        accounts.iter().map(line).collect::<Vec<_>>().join("\n")
    })
}

fn login(app: &App, args: &LoginArgs) -> Result<()> {
    let account = if let Some(user) = &args.gh_user {
        app.services.auth.login_with_gh_account(user)?
    } else if args.with_token {
        app.services.auth.login_with_token(&input::stdin_token()?)?
    } else if args.from_gh {
        app.services.auth.login_with_gh_cli()?
    } else {
        device_flow(app)?
    };
    app.emit(&account, || format!("Signed in as {}", account.login))
}

fn device_flow(app: &App) -> Result<AccountDto> {
    let start = app.services.auth.login_start()?;
    app.note(&format!(
        "Open {} and enter the code: {}",
        start.verification_uri, start.user_code
    ));
    let deadline = Instant::now() + Duration::from_secs(u64::from(start.expires_in));
    let mut interval = u64::from(start.interval.max(1));
    while Instant::now() < deadline {
        thread::sleep(Duration::from_secs(interval));
        match app.services.auth.login_poll(&start.device_code)? {
            LoginPollDto::Authorized { account } => return Ok(account),
            LoginPollDto::SlowDown => interval += 5,
            LoginPollDto::Pending => {}
            LoginPollDto::Expired => bail!("the code expired; run `gitvault auth login` again"),
            LoginPollDto::Denied => bail!("authorization was denied"),
        }
    }
    bail!("timed out waiting for authorization")
}
