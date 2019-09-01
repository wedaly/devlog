//! Load configuration values from environment variables.

use dirs;
use std::env;
use std::path::{Path, PathBuf};

const DEVLOG_REPO_ENV_VAR: &'static str = "DEVLOG_REPO";
const DEFAULT_HOME_DIR: &'static str = "devlogs";
const EDITOR_ENV_VAR: &'static str = "DEVLOG_EDITOR";
const DEFAULT_EDITOR: &'static str = "nano";

pub struct Config {
    repo_dir: PathBuf,
    editor_prog: String,
}

impl Config {
    /// Create a new configuration object with the specified configuration values.
    /// This is used mainly for testing.
    pub fn new(repo_dir: &Path, editor_prog: &str) -> Config {
        Config {
            repo_dir: repo_dir.to_path_buf(),
            editor_prog: editor_prog.to_string(),
        }
    }

    /// Load configuration from environment variables,
    /// providing defaults if the environment variables are not defined.
    pub fn load() -> Config {
        let repo_dir_str = env::var(DEVLOG_REPO_ENV_VAR)
            .ok()
            .unwrap_or_else(default_repo_dir);
        let repo_dir = PathBuf::from(repo_dir_str);

        let editor_prog = env::var(EDITOR_ENV_VAR).unwrap_or(DEFAULT_EDITOR.to_string());

        Config {
            repo_dir,
            editor_prog,
        }
    }

    /// The directory of the user's devlog repository, which may or may not exist.
    /// Defaults to "$HOME/devlogs"
    pub fn repo_dir(&self) -> &Path {
        self.repo_dir.as_path()
    }

    /// The text editor program for editing devlog entry files.
    /// Defaults to "nano".
    pub fn editor_prog(&self) -> &str {
        &self.editor_prog
    }
}

fn default_repo_dir() -> String {
    let mut p = PathBuf::new();
    p.push(dirs::home_dir().expect("Could not find home directory"));
    p.push(DEFAULT_HOME_DIR);
    p.to_string_lossy().to_string()
}
