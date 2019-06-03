use crate::error::Error;
use crate::file::LogFile;
use crate::repository::LogRepository;
use crate::task::{Task, TaskStatus};
use std::io::Write;

#[derive(Debug, Copy, Clone)]
pub enum DisplayMode {
    ShowAll,
    ShowOnly(TaskStatus),
}

impl DisplayMode {
    pub fn show_section_names(&self) -> bool {
        match self {
            DisplayMode::ShowAll => true,
            DisplayMode::ShowOnly(_) => false,
        }
    }

    pub fn show_status(&self, s: &TaskStatus) -> bool {
        match self {
            DisplayMode::ShowAll => true,
            DisplayMode::ShowOnly(status) => s == status,
        }
    }
}

pub fn print<W: Write>(w: &mut W, repo: &LogRepository, d: DisplayMode) -> Result<(), Error> {
    let g = load_tasks_group_by_status(repo)?;
    print_status_report(w, &g, d)
}

fn load_tasks_group_by_status(repo: &LogRepository) -> Result<GroupedTasks, Error> {
    let mut grouped = GroupedTasks::new();
    if let Some(logpath) = repo.latest()? {
        let f = LogFile::load(logpath.path())?;
        f.tasks().iter().for_each(|t| grouped.insert(t));
    }
    Ok(grouped)
}

const ALL_STATUSES: &[TaskStatus] = &[
    TaskStatus::Started,
    TaskStatus::ToDo,
    TaskStatus::Blocked,
    TaskStatus::Done,
];

fn print_status_report<W: Write>(w: &mut W, g: &GroupedTasks, d: DisplayMode) -> Result<(), Error> {
    let mut has_prev = false;
    for status in ALL_STATUSES {
        if d.show_status(status) {
            let tasks = g.retrieve(status);
            if tasks.len() > 0 {
                if has_prev {
                    write!(w, "\n")?;
                }
                print_section(w, status, tasks, d)?;
                has_prev = true;
            }
        }
    }

    if !has_prev {
        write!(w, "[no tasks]\n")?;
    }

    Ok(())
}

fn print_section<W: Write>(
    w: &mut W,
    status: &TaskStatus,
    tasks: &[Task],
    d: DisplayMode,
) -> Result<(), Error> {
    if d.show_section_names() {
        write!(w, "{}:\n", status.display_name())?;
    }
    for t in tasks {
        write!(w, "{}\n", t)?;
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

    fn retrieve(&self, status: &TaskStatus) -> &[Task] {
        match status {
            TaskStatus::ToDo => &self.todo,
            TaskStatus::Started => &self.started,
            TaskStatus::Blocked => &self.blocked,
            TaskStatus::Done => &self.done,
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

    fn check_status(input_tasks: Vec<Task>, display_mode: DisplayMode, expected_status: &str) {
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
        print(&mut buf, &repo, display_mode).unwrap();
        let actual_status = str::from_utf8(&buf).unwrap();
        assert_eq!(actual_status, expected_status);
    }

    #[test]
    fn test_status_no_tasks() {
        check_status(vec![], DisplayMode::ShowAll, "[no tasks]\n");
    }

    #[test]
    fn test_status_only_todo() {
        let tasks = vec![
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::ToDo, "Bar"),
        ];
        check_status(tasks, DisplayMode::ShowAll, "To Do:\n* Foo\n* Bar\n");
    }

    #[test]
    fn test_status_only_started() {
        let tasks = vec![
            Task::new(TaskStatus::Started, "Foo"),
            Task::new(TaskStatus::Started, "Bar"),
        ];
        check_status(tasks, DisplayMode::ShowAll, "In Progress:\n^ Foo\n^ Bar\n")
    }

    #[test]
    fn test_status_only_blocked() {
        let tasks = vec![
            Task::new(TaskStatus::Blocked, "Foo"),
            Task::new(TaskStatus::Blocked, "Bar"),
        ];
        check_status(tasks, DisplayMode::ShowAll, "Blocked:\n- Foo\n- Bar\n");
    }

    #[test]
    fn test_status_only_done() {
        let tasks = vec![
            Task::new(TaskStatus::Done, "Foo"),
            Task::new(TaskStatus::Done, "Bar"),
        ];
        check_status(tasks, DisplayMode::ShowAll, "Done:\n+ Foo\n+ Bar\n");
    }

    #[test]
    fn test_status_todo_and_blocked() {
        let tasks = vec![
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::Blocked, "Bar"),
        ];
        check_status(
            tasks,
            DisplayMode::ShowAll,
            "To Do:\n* Foo\n\nBlocked:\n- Bar\n",
        );
    }

    #[test]
    fn test_status_blocked_and_done() {
        let tasks = vec![
            Task::new(TaskStatus::Blocked, "Bar"),
            Task::new(TaskStatus::Done, "Baz"),
        ];
        check_status(
            tasks,
            DisplayMode::ShowAll,
            "Blocked:\n- Bar\n\nDone:\n+ Baz\n",
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
            DisplayMode::ShowAll,
            "In Progress:\n^ Bar\n\nTo Do:\n* Foo\n\nBlocked:\n- Baz\n\nDone:\n+ Boo\n",
        );
    }

    #[test]
    fn test_show_only_todo() {
        let tasks = vec![
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::Started, "Bar"),
            Task::new(TaskStatus::Blocked, "Baz"),
            Task::new(TaskStatus::Done, "Boo"),
        ];
        check_status(tasks, DisplayMode::ShowOnly(TaskStatus::ToDo), "* Foo\n");
    }

    #[test]
    fn test_show_only_started() {
        let tasks = vec![
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::Started, "Bar"),
            Task::new(TaskStatus::Blocked, "Baz"),
            Task::new(TaskStatus::Done, "Boo"),
        ];
        check_status(tasks, DisplayMode::ShowOnly(TaskStatus::Started), "^ Bar\n");
    }

    #[test]
    fn test_show_only_blocked() {
        let tasks = vec![
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::Started, "Bar"),
            Task::new(TaskStatus::Blocked, "Baz"),
            Task::new(TaskStatus::Done, "Boo"),
        ];
        check_status(tasks, DisplayMode::ShowOnly(TaskStatus::Blocked), "- Baz\n");
    }

    #[test]
    fn test_show_only_done() {
        let tasks = vec![
            Task::new(TaskStatus::ToDo, "Foo"),
            Task::new(TaskStatus::Started, "Bar"),
            Task::new(TaskStatus::Blocked, "Baz"),
            Task::new(TaskStatus::Done, "Boo"),
        ];
        check_status(tasks, DisplayMode::ShowOnly(TaskStatus::Done), "+ Boo\n");
    }
}
