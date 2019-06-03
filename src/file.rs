use crate::task::Task;
use std::fs::File;
use std::io::Error as IOError;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogFile {
    tasks: Vec<Task>,
}

impl LogFile {
    pub fn load(path: &Path) -> Result<LogFile, IOError> {
        let f = File::open(path)?;
        let r = BufReader::new(f);
        let mut tasks = Vec::new();
        for line in r.lines() {
            if let Some(task) = Task::from_string(&line?) {
                tasks.push(task)
            }
        }
        Ok(LogFile { tasks })
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Task, TaskStatus};
    use std::fs::OpenOptions;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("testlog");
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&p)
            .unwrap();
        write!(f, "+ DONE\n").unwrap();
        write!(f, "- BLOCKED\n").unwrap();
        write!(f, "* INCOMPLETE\n").unwrap();
        write!(f, "COMMENT\n").unwrap();

        let lf = LogFile::load(&p).unwrap();
        let expected = vec![
            Task::new(TaskStatus::Done, "DONE"),
            Task::new(TaskStatus::Blocked, "BLOCKED"),
            Task::new(TaskStatus::ToDo, "INCOMPLETE"),
        ];
        assert_eq!(lf.tasks(), &expected[..]);
    }
}
