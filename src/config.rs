use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

pub fn path() -> PathBuf {
    dirs::config_dir()
        .expect("advent of code must be run by a user with a home directory")
        .join("adventofcode")
        .join("config.toml")
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Session cookie
    pub session: String,

    /// Paths are independently configured per year.
    pub paths: HashMap<u32, Paths>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Paths {
    /// Path to input files
    pub input_files: Option<PathBuf>,
}

impl Config {
    pub fn save(&self) -> Result<(), Error> {
        let path = path();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let serialized = toml::ser::to_string_pretty(self)?;
        std::fs::write(path, serialized.as_bytes()).map_err(Into::into)
    }

    pub fn load() -> Result<Self, Error> {
        let data = std::fs::read(path())?;
        toml::de::from_slice(&data).map_err(Into::into)
    }

    fn input_files_inner(&self, year: u32) -> Option<PathBuf> {
        Some(self.paths.get(&year)?.input_files.as_ref()?.to_owned())
    }

    pub fn input_files(&self, year: u32) -> PathBuf {
        match self.input_files_inner(year) {
            Some(input_files) => input_files,
            None => std::env::current_dir()
                .expect("current dir is sane")
                .join("inputs"),
        }
    }

    pub fn input_for(&self, year: u32, day: u8) -> PathBuf {
        self.input_files(year).join(format!("input-{:02}.txt", day))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("configuration could not be loaded")]
    CouldNotLoad(#[from] std::io::Error),
    #[error("malformed configuration")]
    Malformed(#[from] toml::de::Error),
    #[error("failed to serialize")]
    CouldNotSerialize(#[from] toml::ser::Error),
}
