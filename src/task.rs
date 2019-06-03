use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    ToDo,
    Started,
    Done,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    status: TaskStatus,
    content: String,
}

impl Task {
    pub fn new(status: TaskStatus, content: &str) -> Task {
        Task {
            status,
            content: content.to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<Task> {
        let parse_content = |s: &str| s[1..].trim().to_string();
        if s.starts_with("*") {
            Some(Task {
                status: TaskStatus::ToDo,
                content: parse_content(s),
            })
        } else if s.starts_with("^") {
            Some(Task {
                status: TaskStatus::Started,
                content: parse_content(s),
            })
        } else if s.starts_with("+") {
            Some(Task {
                status: TaskStatus::Done,
                content: parse_content(s),
            })
        } else if s.starts_with("-") {
            Some(Task {
                status: TaskStatus::Blocked,
                content: parse_content(s),
            })
        } else {
            None
        }
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.status {
            TaskStatus::ToDo => write!(f, "* ")?,
            TaskStatus::Started => write!(f, "^ ")?,
            TaskStatus::Done => write!(f, "+ ")?,
            TaskStatus::Blocked => write!(f, "- ")?,
        };
        write!(f, "{}", self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_todo() {
        let t = Task::from_string("* INCOMPLETE").expect("Could not parse todo task");
        assert_eq!(t.status(), TaskStatus::ToDo);
        assert_eq!(t.content(), "INCOMPLETE");
    }

    #[test]
    fn test_parse_started() {
        let t = Task::from_string("^ STARTED").expect("Could not parse started task");
        assert_eq!(t.status(), TaskStatus::Started);
        assert_eq!(t.content(), "STARTED");
    }

    #[test]
    fn test_parse_done() {
        let t = Task::from_string("+ Done").expect("Could not parse done task");
        assert_eq!(t.status(), TaskStatus::Done);
        assert_eq!(t.content(), "Done");
    }

    #[test]
    fn test_parse_blocked() {
        let t = Task::from_string("- Blocked").expect("Could not parse blocked task");
        assert_eq!(t.status(), TaskStatus::Blocked);
        assert_eq!(t.content(), "Blocked");
    }

    #[test]
    fn test_parse_ignore() {
        let t = Task::from_string("Comment");
        assert!(t.is_none());
    }

    #[test]
    fn test_parse_ignore_leading_whitespace() {
        let t = Task::from_string("     * COMMENT");
        assert!(t.is_none());
    }

    #[test]
    fn test_trim_whitespace() {
        let t = Task::from_string("+    done      \n").expect("Could not parse task");
        assert_eq!(t.status(), TaskStatus::Done);
        assert_eq!(t.content(), "done");
    }

    #[test]
    fn test_fmt_todo() {
        let t = Task::new(TaskStatus::ToDo, "INCOMPLETE");
        let s = format!("{}", t);
        assert_eq!(s, "* INCOMPLETE");
    }

    #[test]
    fn test_fmt_started() {
        let t = Task::new(TaskStatus::Started, "STARTED");
        let s = format!("{}", t);
        assert_eq!(s, "^ STARTED");
    }

    #[test]
    fn test_fmt_done() {
        let t = Task::new(TaskStatus::Done, "DONE");
        let s = format!("{}", t);
        assert_eq!(s, "+ DONE");
    }

    #[test]
    fn test_fmt_blocked() {
        let t = Task::new(TaskStatus::Blocked, "BLOCKED");
        let s = format!("{}", t);
        assert_eq!(s, "- BLOCKED");
    }
}
