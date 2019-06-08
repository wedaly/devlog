use crate::config::Config;
use crate::error::Error;
use std::ffi::OsStr;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const HOOK_DIR_NAME: &'static str = "hooks";

pub enum HookType {
    BeforeEdit,
    AfterEdit,
    BeforeRollover,
    AfterRollover,
}

impl HookType {
    pub fn name(&self) -> String {
        match self {
            HookType::BeforeEdit => "before-edit",
            HookType::AfterEdit => "after-edit",
            HookType::BeforeRollover => "before-rollover",
            HookType::AfterRollover => "after-rollover",
        }
        .to_string()
    }
}

const ALL_HOOK_TYPES: &[HookType] = &[
    HookType::BeforeEdit,
    HookType::AfterEdit,
    HookType::BeforeRollover,
    HookType::AfterRollover,
];

const HOOK_TEMPLATE: &'static str = "#!/usr/bin/env sh
# To enable this hook, make this file executable.
";

pub fn init_hooks(repo_dir: &Path) -> Result<(), Error> {
    let hook_dir = hook_dir_path(repo_dir);
    create_dir_all(&hook_dir)?;
    for hook_type in ALL_HOOK_TYPES {
        let mut p = hook_dir.clone();
        p.push(hook_type.name());
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&p)
            .unwrap();
        write!(f, "{}", HOOK_TEMPLATE)?;
    }
    Ok(())
}

pub fn execute_hook<W: Write>(
    w: &mut W,
    config: &Config,
    hook_type: &HookType,
    args: &[&OsStr],
) -> Result<(), Error> {
    if let Some(mut cmd) = hook_cmd(config.repo_dir(), hook_type)? {
        let status = cmd.args(args).status()?;
        if !status.success() {
            if let Some(code) = status.code() {
                write!(w, "{} hook exited with status {}\n", hook_type.name(), code)?;
            }
        }
    }
    Ok(())
}

fn hook_cmd(repo_dir: &Path, hook_type: &HookType) -> Result<Option<Command>, Error> {
    let mut p = hook_dir_path(repo_dir);
    p.push(hook_type.name());
    is_valid(&p).map(|valid| if valid { Some(Command::new(&p)) } else { None })
}

fn hook_dir_path(repo_dir: &Path) -> PathBuf {
    let mut p = repo_dir.to_path_buf();
    p.push(HOOK_DIR_NAME);
    p
}

fn is_valid(p: &Path) -> Result<bool, Error> {
    Ok(p.exists() && is_executable(p)?)
}

fn is_executable(p: &Path) -> Result<bool, Error> {
    p.metadata()
        .map(|metadata| {
            let perm = metadata.permissions();
            perm.mode() & 0o111 != 0
        })
        .map_err(From::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, set_permissions, Permissions};
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    fn create_hook_dir(repo_dir: &Path) {
        let mut hook_dir = repo_dir.to_path_buf();
        hook_dir.push(HOOK_DIR_NAME);
        create_dir(&hook_dir).unwrap();
    }

    fn create_hook_file(repo_dir: &Path, hook_type: HookType, executable: bool) {
        let mut p = repo_dir.to_path_buf();
        p.push(HOOK_DIR_NAME);
        p.push(hook_type.name());

        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&p)
            .unwrap();
        write!(f, "#!/usr/bin/env sh\necho 'Hello world!'\n").unwrap();

        if executable {
            set_permissions(&p, Permissions::from_mode(0o555)).unwrap()
        }
    }

    #[test]
    fn test_init_hooks() {
        let repo_dir = tempdir().unwrap();
        init_hooks(repo_dir.path()).unwrap();

        // Initially, all hooks are disabled
        for hook_type in ALL_HOOK_TYPES {
            let result = hook_cmd(repo_dir.path(), hook_type).unwrap();
            assert!(result.is_none());
        }

        // Enable them by updating permissions
        for hook_type in ALL_HOOK_TYPES {
            let mut p = repo_dir.path().to_path_buf();
            p.push(HOOK_DIR_NAME);
            p.push(hook_type.name());
            set_permissions(&p, Permissions::from_mode(0o555)).unwrap()
        }

        // Now all hooks should be enabled and execute successfully
        for hook_type in ALL_HOOK_TYPES {
            let result = hook_cmd(repo_dir.path(), hook_type).unwrap();
            assert!(result.is_some());

            let mut cmd = result.unwrap();
            let status = cmd.status().unwrap();
            assert!(status.success())
        }
    }

    #[test]
    fn test_hook_dir_does_not_exist() {
        let repo_dir = tempdir().unwrap();
        let result = hook_cmd(repo_dir.path(), &HookType::BeforeEdit).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_hook_file_does_not_exist() {
        let repo_dir = tempdir().unwrap();
        create_hook_dir(repo_dir.path());
        let result = hook_cmd(repo_dir.path(), &HookType::BeforeEdit).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_hook_file_is_not_executable() {
        let repo_dir = tempdir().unwrap();
        create_hook_dir(repo_dir.path());
        create_hook_file(repo_dir.path(), HookType::BeforeEdit, false);
        let result = hook_cmd(repo_dir.path(), &HookType::BeforeEdit).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_hook_valid_hook_cmd() {
        let repo_dir = tempdir().unwrap();
        create_hook_dir(repo_dir.path());
        create_hook_file(repo_dir.path(), HookType::BeforeEdit, true);

        let result = hook_cmd(repo_dir.path(), &HookType::BeforeEdit).unwrap();
        assert!(result.is_some());

        let mut cmd = result.unwrap();
        let status = cmd.status().unwrap();
        assert!(status.success())
    }
}
