extern crate clap;
extern crate devlog;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use devlog::{editor, rollover, status, Config, Error, LogRepository};
use std::fs::{create_dir_all, File};
use std::io::{copy, stdout, Write};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const MAIN_INFO: &'static str =
    "Devlog files are created in the directory at $DEVLOG_HOME, which defaults to $HOME/devlogs if not set.";

const EDIT_INFO: &'static str =
    "Uses the editor program $DEVLOG_EDITOR, which defaults to nano if not set.";

fn main() -> Result<(), Error> {
    let m = App::new("devlog")
        .about("Track daily development work")
        .after_help(MAIN_INFO)
        .version(VERSION)
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("edit")
                .about("Edit the most recent devlog file")
                .after_help(EDIT_INFO),
        )
        .subcommand(
            SubCommand::with_name("rollover")
                .about("Create new devlog file with incomplete and blocked tasks"),
        )
        .subcommand(SubCommand::with_name("status").about("Show recent tasks"))
        .subcommand(
            SubCommand::with_name("tail")
                .about("Show recent devlogs")
                .arg(
                    Arg::with_name("limit")
                        .short("n")
                        .long("limit")
                        .takes_value(true)
                        .value_name("LIMIT")
                        .help("Maximum number of log files to display")
                        .default_value("2"),
                ),
        )
        .get_matches();

    let mut w = stdout();
    match m.subcommand() {
        ("edit", Some(_)) => edit_cmd(&mut w),
        ("rollover", Some(_)) => rollover_cmd(&mut w),
        ("status", Some(_)) => status_cmd(&mut w),
        ("tail", Some(m)) => tail_cmd(&mut w, m),
        _ => panic!("No subcommand"),
    }
}

fn repo(config: &Config) -> Result<LogRepository, Error> {
    let repo_dir = config.repo_dir();
    create_dir_all(&repo_dir)?;
    let repo = LogRepository::new(config.repo_dir());
    Ok(repo)
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

fn edit_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    let config = Config::load();
    let r = repo(&config)?;
    match r.latest()? {
        Some(logpath) => editor::open(w, &config, logpath.path()),
        None => r
            .init()
            .and_then(|logpath| editor::open(w, &config, logpath.path())),
    }
}

fn rollover_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    let config = Config::load();
    let r = repo(&config)?;
    let (logpath, count) = rollover::rollover(&r)?;
    write!(w, "Imported {} tasks into {:?}\n", count, logpath.path())?;
    Ok(())
}

fn status_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    let config = Config::load();
    let r = repo(&config)?;
    status::print(w, &r, status::DisplayMode::ShowAll) // TODO
}

fn tail_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let limit = parse_limit_arg(m)?;
    let config = Config::load();
    let r = repo(&config)?;
    let paths = r.tail(limit)?;
    for (i, logpath) in paths.iter().enumerate() {
        if i > 0 {
            write!(w, "\n----------------------\n")?;
        }
        let mut f = File::open(logpath.path())?;
        copy(&mut f, w)?;
    }
    Ok(())
}
