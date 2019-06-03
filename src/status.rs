use crate::error::Error;
use crate::file::LogFile;
use crate::repository::LogRepository;
use crate::task::TaskStatus;
use std::io::Write;

pub fn print<W: Write>(w: &mut W, repo: &LogRepository) -> Result<(), Error> {
    let mut todo = Vec::new();
    let mut blocked = Vec::new();
    let mut done = Vec::new();

    if let Some(logpath) = repo.latest()? {
        let f = LogFile::load(logpath.path())?;
        for t in f.tasks() {
            match t.status() {
                TaskStatus::Incomplete => todo.push(t.clone()),
                TaskStatus::Blocked => blocked.push(t.clone()),
                TaskStatus::Completed => done.push(t.clone()),
            }
        }
    }

    write!(w, "To Do:\n")?;
    if todo.len() > 0 {
        for t in todo.iter() {
            write!(w, "{}\n", t)?;
        }
    } else {
        write!(w, "[empty]\n")?;
    }

    if blocked.len() > 0 {
        write!(w, "\n")?;
        write!(w, "Blocked:\n")?;
        for t in blocked.iter() {
            write!(w, "{}\n", t)?;
        }
    }

    if done.len() > 0 {
        write!(w, "\n")?;
        write!(w, "Done:\n")?;
        for t in done.iter() {
            write!(w, "{}\n", t)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;
    use std::fs::OpenOptions;
    use std::str;
    use tempfile::tempdir;

    fn check_status(input_tasks: Vec<Task>, expected_status: &str) {
        let dir = tempdir().unwrap();
        let repo = LogRepository::new(dir.path());
        let logpath = repo.init().unwrap();
        let mut f = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(logpath.path())
            .unwrap();

        for t in input_tasks.iter() {
            write!(&mut f, "{}\n", t).unwrap();
        }

        let mut buf = Vec::new();
        print(&mut buf, &repo).unwrap();
        let actual_status = str::from_utf8(&buf).unwrap();
        assert_eq!(actual_status, expected_status);
    }

    #[test]
    fn test_status_no_tasks() {
        check_status(vec![], "To Do:\n[empty]\n");
    }

    #[test]
    fn test_status_only_todo() {
        let tasks = vec![
            Task::new(TaskStatus::Incomplete, "Foo"),
            Task::new(TaskStatus::Incomplete, "Bar"),
        ];
        check_status(tasks, "To Do:\n* Foo\n* Bar\n");
    }

    #[test]
    fn test_status_todo_and_blocking() {
        let tasks = vec![
            Task::new(TaskStatus::Incomplete, "Foo"),
            Task::new(TaskStatus::Blocked, "Bar"),
        ];
        check_status(tasks, "To Do:\n* Foo\n\nBlocked:\n- Bar\n");
    }

    #[test]
    fn test_status_blocking_and_done() {
        let tasks = vec![
            Task::new(TaskStatus::Blocked, "Bar"),
            Task::new(TaskStatus::Completed, "Baz"),
        ];
        check_status(
            tasks,
            "To Do:\n[empty]\n\nBlocked:\n- Bar\n\nDone:\n+ Baz\n",
        );
    }

    #[test]
    fn test_status_all_task_types() {
        let tasks = vec![
            Task::new(TaskStatus::Incomplete, "Foo"),
            Task::new(TaskStatus::Blocked, "Bar"),
            Task::new(TaskStatus::Completed, "Baz"),
        ];
        check_status(tasks, "To Do:\n* Foo\n\nBlocked:\n- Bar\n\nDone:\n+ Baz\n");
    }
}
