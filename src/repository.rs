use std::collections::BinaryHeap;
use std::fs::{read_dir, OpenOptions};
use std::io::Error as IOError;
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

    pub fn init(&self) -> Result<PathBuf, IOError> {
        let p = self.head_path();
        OpenOptions::new().write(true).create_new(true).open(&p)?;
        Ok(p)
    }

    pub fn list(&self) -> Result<Vec<PathBuf>, IOError> {
        let entries = read_dir(&self.dir)?;
        let files: Vec<PathBuf> = entries
            .filter_map(|entry_result| entry_result.map(|e| e.path()).ok())
            .collect();
        Ok(files)
    }

    pub fn tail(&self, limit: usize) -> Result<Vec<PathBuf>, IOError> {
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

    pub fn latest(&self) -> Result<Option<PathBuf>, IOError> {
        let mut all = self.list()?;
        let latest = all.drain(..).fold(None, |acc, p| match acc {
            None => Some(p),
            Some(x) => {
                if p > x {
                    Some(p)
                } else {
                    Some(x)
                }
            }
        });
        Ok(latest)
    }

    fn head_path(&self) -> PathBuf {
        self.dir.join("000001")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_files(dir: &Path, names: &[&str]) -> Result<Vec<PathBuf>, IOError> {
        let mut paths = Vec::new();
        for n in names.iter() {
            let p = dir.join(n);
            let mut f = OpenOptions::new().write(true).create(true).open(&p)?;
            write!(f, "+ DONE")?;
            paths.push(p);
        }
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
        let files = vec!["2019-01-01", "2019-02-03"];
        let expected = create_files(dir.path(), &files[..]).unwrap();
        let repo = LogRepository::new(dir.path());
        let mut paths = repo.list().unwrap();
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
        let files = vec!["2019-04-03", "2019-03-02", "2019-02-01"];
        let expected = create_files(dir.path(), &files[..]).unwrap();
        let repo = LogRepository::new(dir.path());
        for i in 0..3 {
            let paths = repo.tail(i).unwrap();
            assert_eq!(paths, &expected[0..i]);
        }
    }

    #[test]
    fn test_tail_large_limit() {
        let dir = tempdir().unwrap();
        let files = vec!["2019-04-03", "2019-03-02", "2019-02-01"];
        let expected = create_files(dir.path(), &files[..]).unwrap();
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
        let files = vec!["2019-04-03", "2019-03-02", "2019-02-01"];
        let paths = create_files(dir.path(), &files[..]).unwrap();
        let repo = LogRepository::new(dir.path());
        let latest = repo.latest().unwrap();
        match latest {
            None => assert!(false),
            Some(p) => assert_eq!(paths[0], p),
        }
    }
}
