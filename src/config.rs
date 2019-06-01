use dirs;
use std::env;
use std::path::{Path, PathBuf};

pub struct Config {
    repo_dir: PathBuf,
}

impl Config {
    pub fn load() -> Config {
        let repo_dir_str = repo_dir_from_env().unwrap_or_else(default_repo_dir);
        let repo_dir = PathBuf::from(repo_dir_str);
        Config { repo_dir }
    }

    pub fn repo_dir(&self) -> &Path {
        self.repo_dir.as_path()
    }
}

fn repo_dir_from_env() -> Option<String> {
    env::var("DEVLOG_HOME").ok()
}

fn default_repo_dir() -> String {
    let mut p = PathBuf::new();
    p.push(dirs::home_dir().expect("Could not find home directory"));
    p.push("devlogs");
    p.to_string_lossy().to_string()
}
