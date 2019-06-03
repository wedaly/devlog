use crate::config::Config;
use crate::error::Error;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub fn open<W: Write>(w: &mut W, config: &Config, path: &Path) -> Result<(), Error> {
    let prog = config.editor_prog();
    let status = Command::new(prog).arg(&path).status()?;

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
