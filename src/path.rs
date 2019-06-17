//! Path to a devlog entry file.

use crate::error::Error;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// The maximum possible sequence number of a devlog entry file.
pub const MAX_SEQ_NUM: usize = 999_999_999;

/// The number of digits in a devlog entry filename.
pub const NUM_DIGITS: usize = 9;

#[derive(Debug, Eq)]
pub struct LogPath {
    path: PathBuf,
    seq_num: usize,
}

/// Devlog entry files are numbered sequentially, starting from one.
/// Each filename is nine digits with the extension ".devlog"; for example, "000000123.devlog".
/// This ensures that the devlog files appear in sequential order when sorted alphabetically.
impl LogPath {
    /// Create a new path with the specified sequence number, which must be at least one
    /// and at most `MAX_SEQ_NUM`.
    pub fn new(dir: &Path, seq_num: usize) -> LogPath {
        assert!(seq_num > 0 && seq_num <= MAX_SEQ_NUM);
        let mut path = dir.to_path_buf();
        path.push(format!("{:09}.devlog", seq_num));
        LogPath { path, seq_num }
    }

    /// Parse the sequence number from a filesystem path.
    /// Returns `None` if the filename isn't formatted like "000000123.devlog".
    pub fn from_path(path: PathBuf) -> Option<LogPath> {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let seq_num: Option<usize> = stem.parse().ok();
        match (stem, ext, seq_num) {
            (s, "devlog", Some(seq_num)) if s.len() == NUM_DIGITS => {
                Some(LogPath { path, seq_num })
            }
            _ => None,
        }
    }

    /// Returns the path for the next entry in the sequence.
    /// In the unlikely event that the maximum sequence number is reached,
    /// returns `Error::LogFileLimitExceeded`.
    pub fn next(&self) -> Result<LogPath, Error> {
        let seq_num = self.seq_num + 1;
        if seq_num > MAX_SEQ_NUM {
            Err(Error::LogFileLimitExceeded)
        } else {
            let mut path = match self.path.parent() {
                Some(p) => p.to_path_buf(),
                None => PathBuf::new(),
            };
            path.push(format!("{:09}.devlog", seq_num));
            Ok(LogPath { path, seq_num })
        }
    }

    /// Returns the sequence number (e.g. "00000123.devlog" would have sequence number 123)
    pub fn seq_num(&self) -> usize {
        self.seq_num
    }

    /// Returns the filesystem path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Order by sequence number.
impl PartialOrd for LogPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Order by sequence number.
impl Ord for LogPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seq_num.cmp(&other.seq_num)
    }
}

/// Equal if and only if the sequence numbers are equal.
impl PartialEq for LogPath {
    fn eq(&self, other: &Self) -> bool {
        self.seq_num == other.seq_num
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> PathBuf {
        From::from(String::from("/foo/bar"))
    }

    fn rootdir() -> PathBuf {
        From::from(String::from("/"))
    }

    #[test]
    fn test_new() {
        let d = dir();
        let p = LogPath::new(&d, 123);
        assert_eq!(p.seq_num(), 123);
        assert_eq!(p.path(), d.join("000000123.devlog"));
    }

    #[test]
    fn test_from_path() {
        let path = dir().join("000000123.devlog");
        let p = LogPath::from_path(path).unwrap();
        assert_eq!(p.seq_num(), 123);
        assert_eq!(p.path(), dir().join("000000123.devlog"));
    }

    #[test]
    fn test_from_path_max_seq_num() {
        let fname = format!("{}.devlog", MAX_SEQ_NUM);
        let path = dir().join(&fname);
        let p = LogPath::from_path(path).unwrap();
        assert_eq!(p.seq_num(), MAX_SEQ_NUM);
        assert_eq!(p.path(), dir().join(&fname));
    }

    #[test]
    fn test_from_path_not_a_number() {
        let path = dir().join("abc123.devlog");
        assert!(LogPath::from_path(path).is_none());
    }

    #[test]
    fn test_from_path_too_few_digits() {
        let path = dir().join("12345678.devlog");
        assert!(LogPath::from_path(path).is_none());
    }

    #[test]
    fn test_from_path_too_many_digits() {
        let path = dir().join("1234567890.devlog");
        assert!(LogPath::from_path(path).is_none());
    }

    #[test]
    fn test_from_path_seq_num_too_large() {
        let fname = format!("{}.devlog", MAX_SEQ_NUM + 1);
        let path = dir().join(&fname);
        assert!(LogPath::from_path(path).is_none());
    }

    #[test]
    fn test_from_path_wrong_ext() {
        let path = dir().join("000000001.csv");
        assert!(LogPath::from_path(path).is_none());
    }

    #[test]
    fn test_next_in_subdir() {
        let d = dir();
        let p = LogPath::new(&d, 123).next().unwrap();
        assert_eq!(p.seq_num(), 124);
        assert_eq!(p.path(), dir().join("000000124.devlog"));
    }

    #[test]
    fn test_next_in_rootdir() {
        let d = rootdir();
        let p = LogPath::new(&d, 123).next().unwrap();
        assert_eq!(p.seq_num(), 124);
        assert_eq!(p.path(), d.join("000000124.devlog"));
    }

    #[test]
    fn test_next_file_limit_exceeded() {
        let d = dir();
        let p = LogPath::new(&d, MAX_SEQ_NUM).next();
        match p {
            Err(Error::LogFileLimitExceeded) => {}
            _ => assert!(false),
        }
    }

    #[test]
    fn test_ordering() {
        let d = dir();
        let p1 = LogPath::new(&d, 1);
        let p2 = LogPath::new(&d, 2);
        let p3 = LogPath::new(&d, 2);
        assert!(p1 < p2);
        assert!(p2 > p1);
        assert!(p2 == p3);
    }
}
