use crate::error::Error;
use crate::file::LogFile;
use crate::repository::LogRepository;
use crate::task::TaskStatus;
use std::io::Write;

pub fn print<W: Write>(w: &mut W, repo: &LogRepository) -> Result<(), Error> {
    let mut todo = Vec::new();
    let mut blocked = Vec::new();
    let mut done = Vec::new();
    let paths = repo.tail(2)?;

    for (i, logpath) in paths.iter().enumerate() {
        let f = LogFile::load(logpath.path())?;
        for t in f.tasks() {
            match t.status() {
                TaskStatus::Incomplete if i == 0 => todo.push(t.clone()),
                TaskStatus::Blocked if i == 0 => blocked.push(t.clone()),
                TaskStatus::Completed => done.push(t.clone()),
                _ => {}
            }
        }
    }

    write!(w, "To Do:\n")?;
    if todo.len() > 0 {
        for t in todo.iter() {
            write!(w, "{}\n", t)?;
        }
    } else {
        write!(w, "[empty]")?;
    }
    write!(w, "\n")?;

    if blocked.len() > 0 {
        write!(w, "Blocked:\n")?;
        for t in blocked.iter() {
            write!(w, "{}\n", t)?;
        }
        write!(w, "\n")?;
    }

    if done.len() > 0 {
        write!(w, "Done:\n")?;
        for t in done.iter() {
            write!(w, "{}\n", t)?;
        }
    }

    Ok(())
}
