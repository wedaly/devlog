use crate::error::Error;
use crate::file::LogFile;
use crate::repository::LogRepository;
use crate::task::{Task, TaskStatus};
use std::io::Write;

pub fn print<W: Write>(w: &mut W, repo: &LogRepository) -> Result<(), Error> {
    let g = load_tasks_group_by_status(repo)?;
    format_status_report(w, &g)
}

fn load_tasks_group_by_status(repo: &LogRepository) -> Result<GroupedTasks, Error> {
    let mut grouped = GroupedTasks::new();
    if let Some(logpath) = repo.latest()? {
        let f = LogFile::load(logpath.path())?;
        f.tasks().iter().for_each(|t| grouped.insert(t));
    }
    Ok(grouped)
}

fn format_status_report<W: Write>(w: &mut W, g: &GroupedTasks) -> Result<(), Error> {
    if g.started.len() > 0 {
        write!(w, "In Progress:\n")?;
        for t in g.started.iter() {
            write!(w, "{}\n", t)?;
        }
        write!(w, "\n")?;
    }

    write!(w, "To Do:\n")?;
    if g.todo.len() > 0 {
        for t in g.todo.iter() {
            write!(w, "{}\n", t)?;
        }
    } else {
        write!(w, "[empty]\n")?;
    }

    if g.blocked.len() > 0 {
        write!(w, "\n")?;
        write!(w, "Blocked:\n")?;
        for t in g.blocked.iter() {
            write!(w, "{}\n", t)?;
        }
    }

    if g.done.len() > 0 {
        write!(w, "\n")?;
        write!(w, "Done:\n")?;
        for t in g.done.iter() {
            write!(w, "{}\n", t)?;
        }
    }

    Ok(())
}

struct GroupedTasks {
    todo: Vec<Task>,
    started: Vec<Task>,
    blocked: Vec<Task>,
    done: Vec<Task>,
}

impl GroupedTasks {
    fn new() -> GroupedTasks {
        GroupedTasks {
            todo: Vec::new(),
            started: Vec::new(),
            blocked: Vec::new(),
            done: Vec::new(),
        }
    }

    fn insert(&mut self, task: &Task) {
        let t = task.clone();
        match t.status() {
            TaskStatus::ToDo => self.todo.push(t),
            TaskStatus::Started => self.started.push(t),
            TaskStatus::Blocked => self.blocked.push(t),
            TaskStatus::Done => self.done.push(t),
        }
    }
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
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::ToDo, "Bar"),
        ];
        check_status(tasks, "To Do:\n* Foo\n* Bar\n");
    }

    #[test]
    fn test_status_only_started() {
        let tasks = vec![
            Task::new(TaskStatus::Started, "Foo"),
            Task::new(TaskStatus::Started, "Bar"),
        ];
        check_status(tasks, "In Progress:\n^ Foo\n^ Bar\n\nTo Do:\n[empty]\n")
    }

    #[test]
    fn test_status_only_blocked() {
        let tasks = vec![
            Task::new(TaskStatus::Blocked, "Foo"),
            Task::new(TaskStatus::Blocked, "Bar"),
        ];
        check_status(tasks, "To Do:\n[empty]\n\nBlocked:\n- Foo\n- Bar\n");
    }

    #[test]
    fn test_status_only_done() {
        let tasks = vec![
            Task::new(TaskStatus::Done, "Foo"),
            Task::new(TaskStatus::Done, "Bar"),
        ];
        check_status(tasks, "To Do:\n[empty]\n\nDone:\n+ Foo\n+ Bar\n");
    }

    #[test]
    fn test_status_todo_and_blocked() {
        let tasks = vec![
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::Blocked, "Bar"),
        ];
        check_status(tasks, "To Do:\n* Foo\n\nBlocked:\n- Bar\n");
    }

    #[test]
    fn test_status_blocked_and_done() {
        let tasks = vec![
            Task::new(TaskStatus::Blocked, "Bar"),
            Task::new(TaskStatus::Done, "Baz"),
        ];
        check_status(
            tasks,
            "To Do:\n[empty]\n\nBlocked:\n- Bar\n\nDone:\n+ Baz\n",
        );
    }

    #[test]
    fn test_status_all_task_types() {
        let tasks = vec![
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::Started, "Bar"),
            Task::new(TaskStatus::Blocked, "Baz"),
            Task::new(TaskStatus::Done, "Boo"),
        ];
        check_status(
            tasks,
            "In Progress:\n^ Bar\n\nTo Do:\n* Foo\n\nBlocked:\n- Baz\n\nDone:\n+ Boo\n",
        );
    }
}
