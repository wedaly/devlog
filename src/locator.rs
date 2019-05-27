use crate::config::Config;
use chrono::offset::TimeZone;
use chrono::{Date, Datelike, Local};
use std::fs::{read_dir, DirEntry};
use std::path::{Path, PathBuf};

pub enum Locator {
    OnDate(Date<Local>),
    MostRecentBeforeDate(Date<Local>),
}

impl Locator {
    pub fn path(&self, c: &Config) -> Option<PathBuf> {
        match self {
            Locator::OnDate(date) => Some(path_for_date(c, date)),
            Locator::MostRecentBeforeDate(date) => find_most_recent_before_date(c, date),
        }
    }
}

fn path_for_date(c: &Config, date: &Date<Local>) -> PathBuf {
    let mut p = c.repository_dir();
    let filename = format!("{:02}-{:02}-{}", date.year(), date.month(), date.day());
    p.push(filename);
    p
}

fn find_most_recent_before_date(c: &Config, date: &Date<Local>) -> Option<PathBuf> {
    let dir = c.repository_dir();
    let mut file_paths: Vec<PathBuf> = match read_dir(dir) {
        Result::Err(_) => Vec::new(),
        Result::Ok(entries) => entries
            .filter_map(|entry_result| match entry_result {
                Result::Ok(entry) => process_dir_entry(&entry, date),
                Result::Err(_) => None,
            })
            .collect(),
    };
    file_paths.sort_unstable();
    file_paths.pop()
}

fn process_dir_entry(entry: &DirEntry, date: &Date<Local>) -> Option<PathBuf> {
    let p = entry.path();
    if p.is_file() {
        match date_from_path(&p) {
            Some(d) if d < *date => Some(p),
            _ => None,
        }
    } else {
        None
    }
}

fn date_from_path(p: &Path) -> Option<Date<Local>> {
    match p.file_name() {
        Some(f) => {
            let fs = f.to_string_lossy();
            let parts: Vec<&str> = fs.splitn(3, "-").collect();
            if parts.len() == 3 {
                let year = parts[0].parse::<i32>();
                let month = parts[1].parse::<u32>();
                let day = parts[2].parse::<u32>();
                match (year, month, day) {
                    (Result::Ok(y), Result::Ok(m), Result::Ok(d)) => Some(Local.ymd(y, m, d)),
                    _ => None,
                }
            } else {
                None
            }
        }
        None => None,
    }
}
