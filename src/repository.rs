//! A devlog repository is a directory containing devlog entry files.

use crate::error::Error;
use crate::path::LogPath;
use std::collections::BinaryHeap;
use std::fs::{create_dir_all, read_dir, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const HELP_MSG: &'static str = "Welcome to your devlog!

You can add tasks below using this format:
* Use an asterisk (*) for each task you want to complete today.
^ Use a caret symbol (^) for tasks that are in progress.
+ Use a plus sign (+) for tasks you completed
- Use a minus sign (-) for tasks that are blocked.

As you work, you can write your questions, thoughts,
and discoveries alongside your tasks.  These will be
stored in your devlog so you can refer to them later.

To edit this devlog:
   devlog edit

To quickly view tasks:
   devlog status

To rollover incomplete tasks to a fresh devlog:
   devlog rollover

To view recent devlogs:
    devlog tail

Please visit https://devlog-cli.org/ for the full user guide.";

/// Represents a devlog repository
pub struct LogRepository {
    dir: PathBuf,
}

impl LogRepository {
    /// Creates a handle to the devlog repository at the specified path.
    /// The directory may or may not exist.
    pub fn new(p: &Path) -> LogRepository {
        let dir = p.to_path_buf();
        LogRepository { dir }
    }

    /// Checks if the repository has been initialized.
    pub fn initialized(&self) -> Result<bool, Error> {
        Ok(self.dir.exists() && self.list()?.len() > 0)
    }

    /// Initializes the repository.
    /// This creates the directory if it does not exist,
    /// as well as the first devlog entry file with sequence number one.
    /// Fails with an `IOError` if the first devlog entry already exists.
    pub fn init(&self) -> Result<LogPath, Error> {
        // Ensure the directory exists
        create_dir_all(&self.dir)?;

        // Create the first logfile
        let p = LogPath::new(&self.dir, 1);
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p.path())?;

        write!(&mut f, "{}\n", HELP_MSG)?;

        Ok(p)
    }

    /// Returns the path to the repository directory,
    /// which may or may not exist.
    pub fn path(&self) -> &Path {
        &self.dir
    }

    /// Returns all paths to devlog entry files in the repository.
    /// The paths are not necessarily ordered.
    pub fn list(&self) -> Result<Vec<LogPath>, Error> {
        let entries = read_dir(&self.dir)?;
        let paths: Vec<LogPath> = entries
            .filter_map(|entry_result| entry_result.ok().and_then(|e| LogPath::from_path(e.path())))
            .collect();
        Ok(paths)
    }

    /// Returns the most recent devlog entry file paths.
    /// `limit` is the maximum number of paths that may be returned.
    pub fn tail(&self, limit: usize) -> Result<Vec<LogPath>, Error> {
        let mut all = self.list()?;
        let mut heap = BinaryHeap::with_capacity(all.len());
        all.drain(..).for_each(|p| heap.push(p));

        let mut result = Vec::with_capacity(limit);
        for _ in 0..limit {
            match heap.pop() {
                Some(p) => result.push(p),
                None => break,
            }
        }

        return Ok(result);
    }

    /// Returns the most recent devlog entry file path,
    /// or `None` if the repository has not yet been initialized.
    pub fn latest(&self) -> Result<Option<LogPath>, Error> {
        let mut all = self.list()?;
        let latest = all.drain(..).max();
        Ok(latest)
    }

    /// Returns the "nth" most recent devlog entry file path.
    /// For example, `n=0` is the most recent entry,
    /// `n=1` is the second most recent entry,
    /// and so on.  Returns `None` if the repository has not yet been initialized.
    pub fn nth_from_latest(&self, n: usize) -> Result<Option<LogPath>, Error> {
        if n == 0 {
            self.latest()
        } else {
            let mut paths = self.tail(n + 1)?;
            let p = paths.drain(..).nth(n);
            Ok(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_files(dir: &Path, count: usize) -> Result<Vec<LogPath>, Error> {
        let mut paths = Vec::new();
        for i in 0..count {
            let seq = i + 1;
            let p = LogPath::new(dir, seq);
            let mut f = OpenOptions::new().write(true).create(true).open(p.path())?;
            write!(f, "+ DONE")?;
            paths.push(p);
        }
        paths.reverse();
        Ok(paths)
    }

    #[test]
    fn test_list_empty_dir() {
        let dir = tempdir().unwrap();
        let repo = LogRepository::new(dir.path());
        let paths = repo.list().unwrap();
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_list_dir() {
        let dir = tempdir().unwrap();
        let mut expected = create_files(dir.path(), 2).unwrap();
        let repo = LogRepository::new(dir.path());
        let mut paths = repo.list().unwrap();
        expected.sort();
        paths.sort();
        assert_eq!(paths, &expected[..]);
    }

    #[test]
    fn test_tail_empty() {
        let dir = tempdir().unwrap();
        let repo = LogRepository::new(dir.path());
        let paths = repo.tail(5).unwrap();
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_tail() {
        let dir = tempdir().unwrap();
        let expected = create_files(dir.path(), 3).unwrap();
        let repo = LogRepository::new(dir.path());
        for i in 0..=3 {
            let paths = repo.tail(i).unwrap();
            assert_eq!(paths, &expected[0..i]);
        }
    }

    #[test]
    fn test_tail_large_limit() {
        let dir = tempdir().unwrap();
        let expected = create_files(dir.path(), 3).unwrap();
        let repo = LogRepository::new(dir.path());
        let paths = repo.tail(100).unwrap();
        assert_eq!(paths, &expected[..]);
    }

    #[test]
    fn test_latest_empty() {
        let dir = tempdir().unwrap();
        let repo = LogRepository::new(dir.path());
        let latest = repo.latest().unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn test_latest() {
        let dir = tempdir().unwrap();
        let paths = create_files(dir.path(), 3).unwrap();
        let repo = LogRepository::new(dir.path());
        let latest = repo.latest().unwrap();
        match latest {
            None => assert!(false),
            Some(p) => assert_eq!(paths[0], p),
        }
    }

    #[test]
    fn test_nth_from_latest_empty_repo() {
        let dir = tempdir().unwrap();
        let repo = LogRepository::new(dir.path());
        assert!(repo.nth_from_latest(0).unwrap().is_none());
    }

    #[test]
    fn test_nth_from_latest() {
        let dir = tempdir().unwrap();
        let paths = create_files(dir.path(), 3).unwrap();
        let repo = LogRepository::new(dir.path());

        assert_eq!(repo.nth_from_latest(0).unwrap().unwrap(), paths[0]);
        assert_eq!(repo.nth_from_latest(1).unwrap().unwrap(), paths[1]);
        assert_eq!(repo.nth_from_latest(2).unwrap().unwrap(), paths[2]);
        assert!(repo.nth_from_latest(3).unwrap().is_none());
        assert!(repo.nth_from_latest(4).unwrap().is_none());
    }
}
