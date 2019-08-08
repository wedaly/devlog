//! Devlog is a command-line tool for tracking your day-to-day software development work.
//!
//! Devlog entries are stored as numbered files in a directory called a repository.
//! Each entry may contain tasks, which each are assigned a status.
//!
//! This library provides a programmatic interface to the functionality of the devlog tool.

pub mod config;
pub mod editor;
pub mod error;
pub mod file;
pub mod hook;
pub mod path;
pub mod repository;
pub mod rollover;
pub mod status;
pub mod task;

pub use config::Config;
pub use error::Error;
pub use file::LogFile;
pub use path::LogPath;
pub use repository::LogRepository;
pub use task::{Task, TaskStatus};
