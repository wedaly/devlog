use crate::config::Config;
use crate::error::Error;
use crate::file::LogFile;
use crate::header::write_header;
use crate::hook::{execute_hook, HookType};
use crate::path::LogPath;
use crate::task::{Task, TaskStatus};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn rollover<W: Write>(
    w: &mut W,
    config: &Config,
    p: &LogPath,
) -> Result<(LogPath, usize), Error> {
    let path = p.path();
    let next = p.next()?;
    let next_path = next.path();

    execute_hook(w, config, &HookType::BeforeRollover, &[path.as_os_str()])?;
    let tasks = load_carryover_tasks(path)?;
    create_new_logfile(&next_path, &tasks)?;
    execute_hook(
        w,
        config,
        &HookType::AfterRollover,
        &[path.as_os_str(), next_path.as_os_str()],
    )?;

    Ok((next, tasks.len()))
}

fn load_carryover_tasks(path: &Path) -> Result<Vec<Task>, Error> {
    let prev = LogFile::load(path)?;
    let mut tasks = Vec::new();
    prev.tasks().iter().for_each(|t| {
        if let TaskStatus::ToDo | TaskStatus::Started | TaskStatus::Blocked = t.status() {
            tasks.push(t.clone());
        }
    });
    Ok(tasks)
}

fn create_new_logfile(next_path: &Path, tasks: &[Task]) -> Result<(), Error> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(next_path)?;

    write_header(&mut f, false)?;

    for t in tasks {
        write!(f, "{}\n", t)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::LogRepository;
    use tempfile::tempdir;

    #[test]
    fn test_rollover() {
        let mut out = Vec::new();
        let dir = tempdir().unwrap();
        let repo = LogRepository::new(dir.path());
        let config = Config::new(dir.path(), "");

        // Initialize the repository, which creates a single
        // logfile with three example tasks.
        repo.init().unwrap();
        let first_logpath = repo.latest().unwrap().unwrap();

        // Rollover, then check that only todo/started/blocked tasks
        // were imported into the new logfile
        let (new_logpath, num_imported) = rollover(&mut out, &config, &first_logpath).unwrap();
        assert_eq!(num_imported, 3);

        // Check tasks in the new logfile
        let logfile = LogFile::load(new_logpath.path()).unwrap();
        let task_statuses: Vec<TaskStatus> = logfile.tasks().iter().map(|t| t.status()).collect();
        assert_eq!(
            task_statuses,
            vec![TaskStatus::ToDo, TaskStatus::Started, TaskStatus::Blocked]
        );

        // Repo should contain two logfiles
        let mut paths = repo.list().unwrap();
        paths.sort();
        assert_eq!(paths, vec![first_logpath, new_logpath]);
    }
}
