use crate::error::Error;
use chrono::Local;
use std::io::Write;

const HELP_MSG: &'static str = "
Welcome to your devlog!

You can add tasks below using this format:
* Use an asterisk (*) for each task you want to comlete today.
+ Use a plus sign (+) for tasks you completed
- Use a minus sign (-) for tasks that are blocked.

Everything else is a note, which the devlog tool will ignore.  Write anything you'd like!

To edit this devlog:
   devlog edit

To quickly view tasks:
   devlog todo
   devlog blocked
   devlog done

To rollover incomplete and blocked tasks to a fresh devlog:
   devlog rollover

For full documentation, go here: TODO";

pub fn write_header<W: Write>(w: &mut W, include_help_msg: bool) -> Result<(), Error> {
    let today = Local::today();
    write!(w, "{}\n", today.format("%Y-%m-%d"))?;

    if include_help_msg {
        write!(w, "{}\n", HELP_MSG)?;
    }

    write!(w, "\n")?;
    Ok(())
}
