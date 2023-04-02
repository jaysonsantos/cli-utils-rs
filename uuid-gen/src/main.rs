use clap::Parser;
use std::default::Default;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
enum Error {
    #[error("invalid version: {0}")]
    InvalidVersion(String),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Copy, Clone, Default)]
enum Version {
    V4,
    #[default]
    V7,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number: u8 = s.parse()?;
        match number {
            4 => Ok(Version::V4),
            7 => Ok(Version::V7),
            _ => Err(Error::InvalidVersion(s.to_string())),
        }
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        match self {
            Version::V4 => "4",
            Version::V7 => "7",
        }
        .to_string()
    }
}
#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(short, long, default_value_t = Version::default())]
    version: Version,
}

fn main() {
    let args = Args::parse();

    let uuid = match args.version {
        Version::V4 => Uuid::new_v4(),
        Version::V7 => Uuid::now_v7(),
    };
    println!("{uuid}");
}
