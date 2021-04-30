use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

/// Path to the configuration file.
pub fn path() -> PathBuf {
    dirs::config_dir()
        .expect("advent of code must be run by a user with a home directory")
        .join("adventofcode")
        .join("config.toml")
}

/// Path to general purpose data directory.
pub fn data() -> PathBuf {
    dirs::data_dir()
        .expect("data directory is discoverable")
        .join("adventofcode")
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Session cookie
    pub session: String,

    /// Paths are independently configured per year.
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    #[serde(default)]
    pub paths: HashMap<u32, Paths>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Paths {
    /// Path to input files
    pub input_files: Option<PathBuf>,

    /// Path to this years's implementation directory
    pub implementation: Option<PathBuf>,

    /// Path to this year's day template files
    pub day_template: Option<PathBuf>,
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

    /// Path to the configured input files directory for `year`.
    ///
    /// If not configured for `year`, returns the "inputs"
    /// sub-folder of that year's implementation directory.
    pub fn input_files(&self, year: u32) -> PathBuf {
        match self.input_files_inner(year) {
            Some(input_files) => input_files,
            None => self.implementation(year).join("inputs"),
        }
    }

    pub fn input_for(&self, year: u32, day: u8) -> PathBuf {
        self.input_files(year).join(format!("input-{:02}.txt", day))
    }

    /// Set the input files directory for `year`.
    pub fn set_input_files(&mut self, year: u32, path: PathBuf) {
        self.paths.entry(year).or_default().input_files = Some(path);
    }

    fn implementation_inner(&self, year: u32) -> Option<PathBuf> {
        Some(self.paths.get(&year)?.implementation.as_ref()?.to_owned())
    }

    /// Path to the implementation directory for `year`.
    ///
    /// If not configured for `year`, returns the current directory.
    ///
    /// Panics if the system cannot locate the current directory, the
    /// current directory does not exist, or other unlikely scenarios.
    pub fn implementation(&self, year: u32) -> PathBuf {
        match self.implementation_inner(year) {
            Some(implementation) => implementation,
            None => std::env::current_dir().expect("current dir is sane"),
        }
    }

    /// Set the implementation directory for `year`.
    pub fn set_implementation(&mut self, year: u32, path: PathBuf) {
        self.paths.entry(year).or_default().implementation = Some(path);
    }

    fn day_template_inner(&self, year: u32) -> Option<PathBuf> {
        Some(self.paths.get(&year)?.day_template.as_ref()?.to_owned())
    }

    /// Path to the template which will be applied for each day for `year`.
    pub fn day_template(&self, year: u32) -> PathBuf {
        match self.day_template_inner(year) {
            Some(day_template) => day_template,
            None => data().join(year.to_string()).join("day-template"),
        }
    }

    /// Set the day template directory for `year`
    pub fn set_day_template(&mut self, year: u32, path: PathBuf) {
        self.paths.entry(year).or_default().day_template = Some(path);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_deserialize() {
        const TOML_DATA: &str = r#"
session = "I'm a session key!"

[paths.1984]
implementation = "/aoc/was/definitely/a/thing"

[paths.5555]
implementation = "/what/future"
input_files = "/what/future/has/inputs"
day_template = "/future/days"
"#;
        let _: Config = toml::de::from_slice(TOML_DATA.as_bytes()).unwrap();
    }

    #[test]
    fn can_deserialize_without_paths() {
        const TOML_DATA: &str = r#"session = "I'm a session key!""#;
        let _: Config = toml::de::from_slice(TOML_DATA.as_bytes()).unwrap();
    }

    #[test]
    fn can_serialize() {
        let mut config = Config::default();
        config.session = "foo bar session session".into();
        config.paths.entry(1984).or_default().implementation = Some("/aoc/was/definitely/a/thing".into());

        {
            let paths = Paths {
                implementation: Some("/what/future".into()),
                input_files: Some("/what/future/has/inputs".into()),
                day_template: Some("/future/days".into()),
            };
            config.paths.insert(1984, paths);
        }

        toml::ser::to_string_pretty(&config).unwrap();
    }
}
