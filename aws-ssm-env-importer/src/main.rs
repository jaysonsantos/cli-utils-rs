use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use clap::Parser as ClapParser;
use color_eyre::eyre::{Result, WrapErr};
use envfile::EnvFile;
use r2d2::Pool;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rusoto_core::{Region, RusotoError};
use rusoto_ssm::{PutParameterError, PutParameterRequest, Ssm, SsmClient as RusotoSsmClient};
use tokio::runtime::{Builder, Runtime};

const MAXIMUM_SSM_CLIENTS: u32 = 4;

#[derive(Debug)]
struct DumbError;

impl Display for DumbError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(f.write_str("Error")?)
    }
}

impl std::error::Error for DumbError {}

#[derive(Debug, ClapParser)]
struct Options {
    #[arg(short = 'f', long = "env-file")]
    env_file: PathBuf,
    #[arg(short = 'r', long = "region")]
    region: String,
    #[arg(short = 'e', long = "environment")]
    environment: String,
    #[arg(short = 'a', long = "app-name")]
    app_name: String,
    #[arg(short = 't', long = "template")]
    /// Template to generate the key on SSM side, example "/{environment}/{app_name}/{key}"
    template: String,
    #[arg(short = 'o', long = "overwrite")]
    overwrite: bool,
    #[arg(short = 'u', long = "uppercase")]
    uppercase: bool,
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,
}

struct SsmConnectionPool(Region);

impl r2d2::ManageConnection for SsmConnectionPool {
    type Connection = RusotoSsmClient;
    type Error = DumbError;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(RusotoSsmClient::new(self.0.clone()))
    }

    fn is_valid(&self, _: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, mut conn: &mut Self::Connection) -> bool {
        self.is_valid(&mut conn).is_err()
    }
}

lazy_static::lazy_static! {
    pub (crate) static ref OPTIONS: Options = Options::parse();
    static ref RUNTIME: Runtime = Builder::new_multi_thread().enable_all().build().unwrap();
}

fn to_template(var: &str) -> String {
    format!("{{{}}}", var)
}

fn format_key(template: &str, key: &str, uppercase: bool, data: &HashMap<&str, &str>) -> String {
    // TODO: This is pretty slow and does not support spaces on the key
    let key = if uppercase {
        key.to_uppercase()
    } else {
        key.to_lowercase()
    };
    let mut output = template.to_owned().replace(&to_template("key"), &key);
    for (key, value) in data {
        output = output.replace(&to_template(key), value);
    }
    output.trim().to_owned()
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let env = EnvFile::new(&OPTIONS.env_file)?;
    let key_template = to_template("key");
    if !OPTIONS.template.contains(&key_template) {
        eprintln!("{{key}} has to be defined in template");
        process::exit(1);
    }

    let mut data = HashMap::new();
    data.insert("environment", OPTIONS.environment.as_str());
    data.insert("app_name", OPTIONS.app_name.as_str());

    let pool = r2d2::Pool::builder()
        .max_size(MAXIMUM_SSM_CLIENTS)
        .build(SsmConnectionPool(Region::from_str(&OPTIONS.region)?))?;

    env.store
        .par_iter()
        .for_each(move |(key, value)| put_parameter(&data, pool.clone(), &key, value));

    Ok(())
}

fn put_parameter(
    data: &HashMap<&str, &str>,
    pool: Pool<SsmConnectionPool>,
    key: &str,
    value: &str,
) {
    let ssm = pool.get().unwrap();
    let normalized_key = format_key(&OPTIONS.template, &key, OPTIONS.uppercase, &data);
    let normalized_value = value.trim();
    if OPTIONS.dry_run {
        println!(
            "Would import '{}' with value '{}' overwrite: {}",
            normalized_key, normalized_value, OPTIONS.overwrite
        );
        return;
    }
    loop {
        let request = PutParameterRequest {
            name: normalized_key.clone(),
            value: normalized_value.to_string(),
            type_: Some("SecureString".to_string()),
            overwrite: Some(OPTIONS.overwrite),
            ..Default::default()
        };
        match RUNTIME.block_on(ssm.put_parameter(request)) {
            Ok(response) => {
                println!(
                    "{} set to version {}",
                    normalized_key,
                    response.version.unwrap()
                );
                break;
            }
            Err(RusotoError::Service(PutParameterError::ParameterAlreadyExists(_))) => {
                println!("Ignored {} because it already exists", normalized_key);
                break;
            }
            Err(RusotoError::Unknown(ref e))
                if String::from_utf8_lossy(e.body.as_ref()).contains("ThrottlingException") =>
            {
                thread::sleep(Duration::from_secs(1));
            }
            error @ Err(_) => {
                error
                    .wrap_err_with(|| {
                        format!(
                            "Unexpected error while trying to put key = {:?} and value = {:?}",
                            normalized_key, normalized_value
                        )
                    })
                    .unwrap();
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{format_key, to_template};

    #[test]
    fn test_format_key() {
        let mut data = HashMap::new();
        data.insert("environment", "staging");
        data.insert("app_name", "app");

        let formatted = format_key("{key}", "test", false, &HashMap::new());
        assert_eq!(formatted, "test");

        let formatted = format_key("/{environment}/{key}", "test", false, &HashMap::new());
        assert_eq!(formatted, "/{environment}/test");

        let formatted = format_key("/{environment}/{app_name}/{key}", "test", false, &data);
        assert_eq!(formatted, "/staging/app/test");

        let formatted = format_key("{key}", "test", true, &HashMap::new());
        assert_eq!(formatted, "TEST");

        let formatted = format_key("/{environment}/{key}", "test", true, &HashMap::new());
        assert_eq!(formatted, "/{environment}/TEST");

        let formatted = format_key("/{environment}/{app_name}/{key}", "test", true, &data);
        assert_eq!(formatted, "/staging/app/TEST");
    }

    #[test]
    fn test_to_template() {
        assert_eq!(to_template("key"), "{key}");
    }
}
