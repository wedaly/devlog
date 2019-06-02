use dirs;
use std::env;
use std::path::{Path, PathBuf};

const DEVLOG_HOME_ENV_VAR: &'static str = "DEVLOG_HOME";
const DEFAULT_HOME_DIR: &'static str = "devlogs";

pub struct Config {
    repo_dir: PathBuf,
}

impl Config {
    pub fn load() -> Config {
        let repo_dir_str = env::var(DEVLOG_HOME_ENV_VAR)
            .ok()
            .unwrap_or_else(default_repo_dir);
        let repo_dir = PathBuf::from(repo_dir_str);
        Config { repo_dir }
    }

    pub fn repo_dir(&self) -> &Path {
        self.repo_dir.as_path()
    }
}

fn default_repo_dir() -> String {
    let mut p = PathBuf::new();
    p.push(dirs::home_dir().expect("Could not find home directory"));
    p.push(DEFAULT_HOME_DIR);
    p.to_string_lossy().to_string()
}
