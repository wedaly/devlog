extern crate clap;
extern crate devlog;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use devlog::{editor, rollover, status, Config, Error, LogRepository, TaskStatus};
use std::fs::{create_dir_all, File};
use std::io::{copy, stdin, stdout, Write};
use std::process::exit;

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
            SubCommand::with_name("init")
                .about("Initialize a new devlog repository if it does not already exist."),
        )
        .subcommand(
            SubCommand::with_name("edit")
                .about("Edit the most recent devlog file")
                .after_help(EDIT_INFO),
        )
        .subcommand(
            SubCommand::with_name("rollover")
                .about("Create new devlog file with incomplete and blocked tasks"),
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("Show recent tasks")
                .arg(
                    Arg::with_name("show")
                        .short("s")
                        .long("show")
                        .takes_value(true)
                        .value_name("SHOW")
                        .possible_values(&["all", "todo", "started", "blocked", "done"])
                        .default_value("all")
                        .help("Sections to show"),
                )
                .arg(
                    Arg::with_name("back")
                        .short("b")
                        .long("back")
                        .takes_value(true)
                        .value_name("BACK")
                        .default_value("0")
                        .help("Show tasks from a previous devlog"),
                ),
        )
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
        ("init", Some(_)) => init_cmd(&mut w),
        ("edit", Some(_)) => edit_cmd(&mut w),
        ("rollover", Some(_)) => rollover_cmd(&mut w),
        ("status", Some(m)) => status_cmd(&mut w, m),
        ("tail", Some(m)) => tail_cmd(&mut w, m),
        _ => panic!("No subcommand"),
    }
}

fn prompt_confirm<W: Write>(w: &mut W, msg: &str) -> Result<bool, Error> {
    write!(w, "{} [y/n] ", msg)?;
    w.flush()?;

    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .map(|_| {
            let s = input.trim().to_lowercase();
            s == "yes" || s == "y"
        })
        .map_err(From::from)
}

fn open_or_create_repo<W: Write>(
    w: &mut W,
    config: &Config,
) -> Result<(LogRepository, bool), Error> {
    let mut created = false;
    let dir_path = config.repo_dir();
    if !dir_path.exists() {
        let msg = format!("Create devlog repository at {:?}?", dir_path);
        if prompt_confirm(w, &msg)? {
            created = true;
            create_dir_all(dir_path)?;
        } else {
            exit(1);
        }
    }

    if !dir_path.is_dir() {
        return Err(Error::InvalidArgError("Repository path is not a directory"));
    }

    Ok((LogRepository::new(dir_path), created))
}

fn init_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    let config = Config::load();
    let (repo, created) = open_or_create_repo(w, &config)?;
    if created {
        repo.init()?;
        write!(w, "Initialized devlog repository at {:?}\n", repo.path())?;
    } else {
        write!(w, "Devlog repository already exists at {:?}\n", repo.path())?;
    }
    Ok(())
}

fn edit_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    let config = Config::load();
    let (r, _) = open_or_create_repo(w, &config)?;
    match r.latest()? {
        Some(logpath) => editor::open(w, &config, logpath.path()),
        None => r
            .init()
            .and_then(|logpath| editor::open(w, &config, logpath.path())),
    }
}

fn rollover_cmd<W: Write>(w: &mut W) -> Result<(), Error> {
    let config = Config::load();
    let r = LogRepository::new(config.repo_dir());
    let (logpath, count) = rollover::rollover(&r)?;
    write!(w, "Imported {} tasks into {:?}\n", count, logpath.path())?;
    Ok(())
}

fn status_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let num_back = m
        .value_of("back")
        .unwrap()
        .parse::<usize>()
        .map_err(|_| Error::InvalidArgError("back must be an integer"))?;

    let display_mode = match m.value_of("show") {
        Some("all") => status::DisplayMode::ShowAll,
        Some("todo") => status::DisplayMode::ShowOnly(TaskStatus::ToDo),
        Some("started") => status::DisplayMode::ShowOnly(TaskStatus::Started),
        Some("blocked") => status::DisplayMode::ShowOnly(TaskStatus::Blocked),
        Some("done") => status::DisplayMode::ShowOnly(TaskStatus::Done),
        _ => panic!("Invalid value for show arg"),
    };

    let config = Config::load();
    let r = LogRepository::new(config.repo_dir());
    status::print(w, &r, num_back, display_mode)
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

fn tail_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let limit = parse_limit_arg(m)?;
    let config = Config::load();
    let r = LogRepository::new(config.repo_dir());
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
