use crate::error::Error;
use std::env;
use std::io::Write;
use std::path::Path;
use std::process::Command;

const EDITOR_ENV_VAR: &'static str = "DEVLOG_EDITOR";
const DEFAULT_EDITOR: &'static str = "nano";

pub fn open<W: Write>(w: &mut W, path: &Path) -> Result<(), Error> {
    let prog = env::var(EDITOR_ENV_VAR).unwrap_or(DEFAULT_EDITOR.to_string());

    let status = Command::new(&prog).arg(&path).status()?;

    if status.success() {
        Ok(())
    } else {
        match status.code() {
            Some(code) => write!(
                w,
                "Command `{} {}` exited with status {}\n",
                prog,
                path.to_string_lossy(),
                code
            )
            .map_err(From::from),
            None => write!(w, "Process terminated by signal\n").map_err(From::from),
        }
    }
}
