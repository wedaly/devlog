use crate::error::Error;
use chrono::Local;
use std::io::Write;

const HELP_MSG: &'static str = "
Welcome to your devlog!

You can add tasks below using this format:
* Use an asterisk (*) for each task you want to complete today.
^ Use a caret symbol (^) for each task that's in progress.
+ Use a plus sign (+) for tasks you completed
- Use a minus sign (-) for tasks that are blocked.

As you work, you can write your questions, thoughts, and discoveries alongside your tasks.
These will be stored in your devlog so you can refer to them later.

To edit this devlog:
   devlog edit

To quickly view tasks:
   devlog status

To rollover incomplete tasks to a fresh devlog:
   devlog rollover

To view recent devlogs:
    devlog tail

Please visit https://devlog-cli.org/ for the full user guide.";

pub fn write_header<W: Write>(w: &mut W, include_help_msg: bool) -> Result<(), Error> {
    let today = Local::today();
    write!(w, "{}\n", today.format("%Y-%m-%d"))?;

    if include_help_msg {
        write!(w, "{}\n", HELP_MSG)?;
    }

    write!(w, "\n")?;
    Ok(())
}
