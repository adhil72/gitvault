use crate::{
    cli::UnlockArgs,
    commands::print_sync_outcome,
    drivers::{store::prompt_confirmed_password, PasswordOptions, Prerequisites, VaultStore},
    error::AppError,
    ui,
};

pub fn unlock(args: UnlockArgs, pw_opts: PasswordOptions) -> Result<(), AppError> {
    Prerequisites::ensure_ready()?;
    let store = VaultStore::open_default()?;
    if !store.is_initialized() {
        store.ensure_initialized_from_existing()?;
    }
    let key = store.unlock(
        pw_opts,
        args.recovery_key.as_deref(),
        args.change_password || args.recovery_key.is_some(),
    )?;
    if args.change_password {
        if pw_opts.non_interactive {
            return Err(AppError::Validation(
                "cannot interactively enter new password in --non-interactive mode".into(),
            ));
        }
        let new_pw = prompt_confirmed_password("Choose a new master password:")?;
        let mut header = store.read_header()?;
        header.change_password(&key, &new_pw)?;
        store.write_header(&header)?;
        let res = store.commit_and_sync("vault: change master password")?;
        print_sync_outcome(res, "Master password updated");
        return Ok(());
    }
    ui::success("Vault unlocked (local session key cached)");
    Ok(())
}

pub fn lock() -> Result<(), AppError> {
    let store = VaultStore::open_default()?;
    if store.lock()? {
        ui::success("Vault locked and local session key wiped");
    } else {
        ui::info("Vault is already locked");
    }
    Ok(())
}
