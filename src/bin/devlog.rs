extern crate clap;
extern crate devlog;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use devlog::{editor, rollover, Config, Error, LogFile, LogRepository, TaskStatus};
use std::fs::File;
use std::io::{copy, stdout, Write};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Error> {
    let limit_arg = Arg::with_name("limit")
        .short("n")
        .long("limit")
        .takes_value(true)
        .value_name("LIMIT")
        .help("Maximum number of log files to display")
        .default_value("2");

    let m = App::new("devlog")
        .about("Track daily development work")
        .version(VERSION)
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("tail")
                .about("Show recent devlog files")
                .arg(limit_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("done")
                .about("Show recently completed tasks")
                .arg(limit_arg.clone()),
        )
        .subcommand(SubCommand::with_name("todo").about("Show incomplete tasks"))
        .subcommand(SubCommand::with_name("blocked").about("Show blocked tasks"))
        .subcommand(SubCommand::with_name("edit").about("Edit the most recent devlog file"))
        .subcommand(
            SubCommand::with_name("rollover")
                .about("Create new devlog file with incomplete and blocked tasks"),
        )
        .get_matches();

    let mut w = stdout();
    match m.subcommand() {
        ("done", Some(m)) => done_cmd(&mut w, m),
        ("todo", Some(_)) => todo_cmd(&mut w),
        ("blocked", Some(_)) => blocked_cmd(&mut w),
        ("edit", Some(_)) => edit_cmd(&mut w),
        ("rollover", Some(_)) => rollover_cmd(&mut w),
        ("tail", Some(m)) => tail_cmd(&mut w, m),
        _ => panic!("No subcommand"),
    }
}

fn repo() -> LogRepository {
    let config = Config::load();
    LogRepository::new(config.repo_dir())
}

fn parse_limit_arg(m: &ArgMatches) -> Result<usize, Error> {
    let limit = m
        .value_of("limit")
        .unwrap()
        .parse::<usize>()
        .map_err(|_| Error::InvalidArgError("limit must be an integer"))?;
    if limit < 1 {
        Err(Error::InvalidArgError("limit must be >= 1"))
    } else {
        Ok(limit)
    }
}

fn done_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let limit = parse_limit_arg(m)?;
    for logpath in repo().tail(limit)? {
        let f = LogFile::load(logpath.path())?;
        for t in f.tasks() {
            if let TaskStatus::Completed = t.status() {
                write!(w, "{}\n", t)?;
            }
        }
    }
    Ok(())
}

fn todo_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    if let Some(logpath) = repo().latest()? {
        let f = LogFile::load(logpath.path())?;
        for t in f.tasks() {
            if let TaskStatus::Incomplete = t.status() {
                write!(w, "{}\n", t)?;
            }
        }
    }
    Ok(())
}

fn blocked_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    if let Some(logpath) = repo().latest()? {
        let f = LogFile::load(logpath.path())?;
        for t in f.tasks() {
            if let TaskStatus::Blocked = t.status() {
                write!(w, "{}\n", t)?;
            }
        }
    }
    Ok(())
}

fn edit_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    let r = repo();
    match r.latest()? {
        Some(logpath) => editor::open(w, logpath.path()),
        None => r.init().and_then(|logpath| editor::open(w, logpath.path())),
    }
}

fn rollover_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    let (logpath, count) = rollover::rollover(&repo())?;
    write!(w, "Imported {} tasks into {:?}\n", count, logpath.path())?;
    Ok(())
}

fn tail_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let limit = parse_limit_arg(m)?;
    for logpath in repo().tail(limit)? {
        write!(w, "{:09}\n", logpath.seq_num())?;
        let mut f = File::open(logpath.path())?;
        copy(&mut f, w)?;
        write!(w, "\n")?;
    }
    Ok(())
}
