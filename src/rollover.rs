use crate::error::Error;
use crate::file::LogFile;
use crate::header::write_header;
use crate::path::LogPath;
use crate::repository::LogRepository;
use crate::task::{Task, TaskStatus};
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
        if let TaskStatus::ToDo | TaskStatus::Blocked = t.status() {
            tasks.push(t.clone());
        }
    });
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rollover_empty_repo() {
        let dir = tempdir().unwrap();
        let repo = LogRepository::new(dir.path());
        let (logpath, num_imported) = rollover(&repo).unwrap();
        assert_eq!(num_imported, 0);

        // newly created logfile contains example tasks
        let logfile = LogFile::load(logpath.path()).unwrap();
        assert_eq!(logfile.tasks().len(), 4);

        // new logfile is only one in the repo
        let paths = repo.list().unwrap();
        assert_eq!(paths, vec![logpath]);
    }

    #[test]
    fn test_rollover() {
        let dir = tempdir().unwrap();
        let repo = LogRepository::new(dir.path());

        // Initialize the repository, which creates a single
        // logfile with three example tasks.
        repo.init().unwrap();
        let first_logpath = repo.latest().unwrap().unwrap();

        // Rollover, then check that only blocked/incomplete tasks
        // were imported into the new logfile
        let (new_logpath, num_imported) = rollover(&repo).unwrap();
        assert_eq!(num_imported, 2);

        // Check tasks in the new logfile
        let logfile = LogFile::load(new_logpath.path()).unwrap();
        let task_statuses: Vec<TaskStatus> = logfile.tasks().iter().map(|t| t.status()).collect();
        assert_eq!(task_statuses, vec![TaskStatus::ToDo, TaskStatus::Blocked]);

        // Repo should contain two logfiles
        let mut paths = repo.list().unwrap();
        paths.sort();
        assert_eq!(paths, vec![first_logpath, new_logpath]);
    }
}
