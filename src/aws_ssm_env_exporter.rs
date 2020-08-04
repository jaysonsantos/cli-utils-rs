use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use color_eyre::eyre::Result;
use envfile::EnvFile;
use log::Level::Debug;
use log::{debug, log_enabled};
use regex::Regex;
use rusoto_core::Region;
use rusoto_ssm::{GetParametersByPathRequest, Ssm, SsmClient};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short = "f", long = "env-file")]
    env_file: PathBuf,
    #[structopt(short = "r", long)]
    region: Option<Region>,
    #[structopt(short = "p", long)]
    path: String,
    #[structopt(short = "e", long)]
    search: Regex,
    #[structopt(short = "t", long)]
    replace: String,
    #[structopt(short = "u", long)]
    uppercase: bool,
    #[structopt(short = "l", long)]
    lowercase: bool,
}

impl Options {
    pub fn get_region(&self) -> Region {
        self.region.clone().unwrap_or_else(Region::default)
    }
}

lazy_static::lazy_static! {
    pub (crate) static ref OPTIONS: Options = Options::from_args();
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let file = File::create(&OPTIONS.env_file)?;
    drop(file);

    let mut env = EnvFile::new(&OPTIONS.env_file)?;

    let cli = SsmClient::new(OPTIONS.get_region());
    let configs = fetch_configs(&cli, None)?;
    for (key, value) in configs {
        let key = transform_key(&key, &OPTIONS);
        env.update(&key, &value);
    }
    env.write()?;
    Ok(())
}

fn fetch_configs(cli: &SsmClient, next_token: Option<String>) -> Result<HashMap<String, String>> {
    let mut output = HashMap::new();
    let request = GetParametersByPathRequest {
        path: OPTIONS.path.clone(),
        with_decryption: Some(true),
        next_token,
        ..Default::default()
    };
    let response = cli.get_parameters_by_path(request).sync()?;
    if let Some(parameters) = response.parameters {
        for parameter in parameters {
            output.insert(parameter.name.unwrap(), parameter.value.unwrap());
        }
    }

    if response.next_token.is_some() {
        output.extend(fetch_configs(cli, response.next_token)?)
    }

    Ok(output)
}

fn transform_key(key: &str, options: &Options) -> String {
    let mut new_key = options
        .search
        .replace(key, options.replace.as_str())
        .to_string();
    if options.lowercase {
        new_key = new_key.to_lowercase();
    }
    if options.uppercase {
        new_key = new_key.to_uppercase();
    }
    if log_enabled!(Debug) {
        debug!(
            "Regex {:?} matches with {:?}? {:?}",
            options.search,
            key,
            options.search.is_match(key)
        );
        debug!("Transform key {:?} to {:?}", key, new_key);
    }
    new_key
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::transform_key;
    use super::Options;

    #[test]
    fn test_transform_key() {
        let re = Regex::new(".+/(.*)$").unwrap();
        let key = "/test/abc/variable";

        let paylodads = vec![
            (
                Options {
                    search: re.clone(),
                    replace: "$1".to_string(),
                    uppercase: true,
                    lowercase: false,
                    env_file: "/dev/null".into(),
                    region: None,
                    path: "".to_string(),
                },
                "VARIABLE",
            ),
            (
                Options {
                    search: re,
                    replace: "$1".to_string(),
                    uppercase: false,
                    lowercase: true,
                    env_file: "/dev/null".into(),
                    region: None,
                    path: "".to_string(),
                },
                "variable",
            ),
        ];

        for (options, expected) in paylodads {
            assert_eq!(transform_key(key, &options), expected);
        }
    }
}
