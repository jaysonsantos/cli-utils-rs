use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use envfile::EnvFile;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rusoto_core::{Region, RusotoError};
use rusoto_ssm::{PutParameterError, PutParameterRequest, Ssm, SsmClient as RusotoSsmClient};
use structopt::StructOpt;

const MAXIMUM_SSM_CLIENTS: u32 = 4;

#[derive(Debug)]
struct DumbError;
impl Display for DumbError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(f.write_str("Error")?)
    }
}
impl std::error::Error for DumbError {}

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short = "f", long = "env-file")]
    env_file: PathBuf,
    #[structopt(short = "r", long = "region")]
    region: String,
    #[structopt(short = "e", long = "environment")]
    environment: String,
    #[structopt(short = "a", long = "app-name")]
    app_name: String,
    #[structopt(short = "t", long = "template")]
    /// Template to generate the key on SSM side, example "/{environment}/{app_name}/{key}"
    template: String,
    #[structopt(short = "o", long = "overwrite")]
    overwrite: bool,
    #[structopt(short = "u", long = "uppercase")]
    uppercase: bool,
    #[structopt(short = "d", long = "dry-run")]
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
    pub (crate) static ref OPTIONS: Options = Options::from_args();
}

fn to_template(var: &str) -> String {
    format!("{{{}}}", var)
}

fn format_key(
    template: &str,
    key: &str,
    uppercase: bool,
    data: &HashMap<String, String>,
) -> String {
    // TODO: This is pretty slow and does not support spaces on the key
    let key = match uppercase {
        true => key.to_uppercase(),
        false => key.to_lowercase(),
    };
    let mut output = template.to_owned().replace(&to_template("key"), &key);
    for (key, value) in data {
        output = output.replace(&to_template(key), value);
    }
    output.trim().to_owned()
}

fn main() -> Result<(), failure::Error> {
    let env = EnvFile::new(&OPTIONS.env_file)?;
    let key_template = to_template("key");
    if OPTIONS.template.contains(&key_template) == false {
        eprintln!("{{key}} has to be defined in template");
        process::exit(1);
    }

    let mut data = HashMap::new();
    data.insert("environment".to_owned(), OPTIONS.environment.clone());
    data.insert("app_name".to_owned(), OPTIONS.app_name.clone());

    let pool = r2d2::Pool::builder()
        .max_size(MAXIMUM_SSM_CLIENTS)
        .build(SsmConnectionPool(Region::from_str(&OPTIONS.region)?))?;

    env.store.par_iter().for_each(move |(key, value)| {
        let ssm = pool.get().unwrap();
        let normalized_key = format_key(&OPTIONS.template, &key, OPTIONS.uppercase, &data);
        let normalized_value = value.trim();
        if OPTIONS.dry_run {
            println!(
                "Would import '{}' to '{}' overwrite: {}",
                normalized_key, normalized_value, OPTIONS.overwrite
            );
            return;
        }
        loop {
            let request = PutParameterRequest {
                name: normalized_key.clone(),
                value: normalized_value.to_string(),
                type_: "SecureString".to_string(),
                overwrite: Some(OPTIONS.overwrite),
                ..Default::default()
            };
            match ssm.put_parameter(request).sync() {
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
                    if String::from_utf8_lossy(e.body.as_slice())
                        .contains("ThrottlingException") =>
                {
                    thread::sleep(Duration::from_secs(1));
                }
                error => {
                    error.unwrap();
                }
            };
        }
    });

    Ok(())
}
