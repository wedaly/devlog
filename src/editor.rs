use std::env;
use std::io::Error as IOError;
use std::io::Write;
use std::path::Path;
use std::process::Command;

const DEFAULT_EDITOR: &'static str = "nano";

pub fn open_in_editor<W: Write>(w: &mut W, path: &Path) -> Result<(), IOError> {
    let prog = env::var("DEVLOG_EDITOR").unwrap_or(DEFAULT_EDITOR.to_string());

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
            ),
            None => write!(w, "Process terminated by signal\n"),
        }
    }
}
