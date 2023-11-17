use crate::config::Config;
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

/// Generate the puzzle URL for a given day
pub fn url_for_day(year: u32, day: u8) -> String {
    format!("https://adventofcode.com/{}/day/{}", year, day)
}

/// Generate the input URL for a given day
pub fn input_url_for_day(year: u32, day: u8) -> String {
    format!("{}/input", url_for_day(year, day))
}

fn throttle(config: &Config) -> Result<(), Error> {
    let path = config.throttle_file();
    let Ok(contents) = std::fs::read_to_string(path) else {
        // if we can't read the file, most likely the issue is that it doesn't exist;
        // in that case, we're fine to download
        return Ok(());
    };
    let Ok(dl_available) = OffsetDateTime::parse(contents.trim(), &Rfc3339) else {
        // malformed file? we solve the issue by trying again
        return Ok(());
    };
    if OffsetDateTime::now_utc() < dl_available {
        let dl_available = dl_available
            .format(&Rfc3339)
            .unwrap_or("unrenderable time".into());
        Err(Error::Throttled(dl_available))
    } else {
        Ok(())
    }
}

/// best effort to update the throttle file
///
/// On failure, just abort
fn update_throttle_file(config: &Config) {
    let path = config.throttle_file();
    let dl_available = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
        + Duration::seconds(900); // 15 minutes
    let Ok(dl_available) = dl_available.format(&Rfc3339) else {
        return;
    };
    let _ = std::fs::write(path, dl_available);
}

/// Download the day's input file
///
/// If the file already exists, silently does nothing. This prevents server spam.
///
/// If the file does not exist, throttles server requests to once every 15 minutes.
pub fn get_input(config: &Config, year: u32, day: u8) -> Result<(), Error> {
    let input_path = config.input_for(year, day);
    if input_path.exists() {
        return Ok(());
    }

    throttle(config)?;

    let client = reqwest::blocking::Client::builder()
        .user_agent("https://github.com/coriolinus/aoclib")
        .gzip(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(Error::ClientBuilder)?;

    let mut response = client
        .get(input_url_for_day(year, day))
        .header(
            reqwest::header::COOKIE,
            format!("session={}", config.session),
        )
        .send()
        .map_err(Error::RequestingInput)?
        .error_for_status()
        .map_err(Error::ResponseStatus)?;

    if let Some(parent) = input_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut file = std::fs::File::create(input_path)?;
    response.copy_to(&mut file).map_err(Error::Downloading)?;

    update_throttle_file(config);

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("building request client")]
    ClientBuilder(#[source] reqwest::Error),
    #[error("requesting input file")]
    RequestingInput(#[source] reqwest::Error),
    #[error("response status unsuccessful")]
    ResponseStatus(#[source] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("downloading to local file")]
    Downloading(#[source] reqwest::Error),
    #[error("download throttled; next available DL {0}")]
    Throttled(String),
}
