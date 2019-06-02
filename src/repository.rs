use crate::error::Error;
use crate::path::LogPath;
use std::collections::BinaryHeap;
use std::fs::{read_dir, OpenOptions};
use std::path::{Path, PathBuf};

pub struct LogRepository {
    dir: PathBuf,
}

impl LogRepository {
    pub fn new(dir: &Path) -> LogRepository {
        LogRepository {
            dir: dir.to_path_buf(),
        }
    }

    pub fn init(&self) -> Result<LogPath, Error> {
        let p = LogPath::new(&self.dir, 1);
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p.path())?;
        Ok(p)
    }

    pub fn list(&self) -> Result<Vec<LogPath>, Error> {
        let entries = read_dir(&self.dir)?;
        let paths: Vec<LogPath> = entries
            .filter_map(|entry_result| entry_result.ok().and_then(|e| LogPath::from_path(e.path())))
            .collect();
        Ok(paths)
    }

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

    pub fn latest(&self) -> Result<Option<LogPath>, Error> {
        let mut all = self.list()?;
        let latest = all.drain(..).fold(None, |acc, p| match acc {
            None => Some(p),
            Some(q) => {
                if p > q {
                    Some(p)
                } else {
                    Some(q)
                }
            }
        });
        Ok(latest)
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
            let p = LogPath::new(dir, i);
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
}
