use crate::error::Error;
use crate::file::LogFile;
use crate::path::LogPath;
use crate::repository::LogRepository;
use crate::task::{Task, TaskStatus};
use crate::header::write_header;
use std::fs::OpenOptions;
use std::io::Write;

pub fn rollover(repo: &LogRepository) -> Result<(LogPath, usize), Error> {
    match repo.latest()? {
        Some(latest) => rollover_from(latest),
        None => repo.init().map(|p| (p, 0)),
    }
}

fn rollover_from(p: LogPath) -> Result<(LogPath, usize), Error> {
    let tasks = load_carryover_tasks(&p)?;
    let next = p.next()?;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(next.path())?;

    write_header(&mut f, false)?;

    for t in tasks.iter() {
        write!(f, "{}\n", t)?;
    }

    Ok((next, tasks.len()))
}

fn load_carryover_tasks(p: &LogPath) -> Result<Vec<Task>, Error> {
    let prev = LogFile::load(p.path())?;
    let mut tasks = Vec::new();
    prev.tasks().iter().for_each(|t| {
        if let TaskStatus::Incomplete | TaskStatus::Blocked = t.status() {
            tasks.push(t.clone());
        }
    });
    Ok(tasks)
}
