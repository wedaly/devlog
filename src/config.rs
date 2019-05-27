use dirs;
use std::env;
use std::path::PathBuf;

pub struct Config {
    dir: String,
}

impl Config {
    pub fn load() -> Config {
        let dir = repo_dir_from_env().unwrap_or_else(default_repo_dir);
        Config { dir }
    }

    pub fn repository_dir(&self) -> PathBuf {
        PathBuf::from(&self.dir)
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
