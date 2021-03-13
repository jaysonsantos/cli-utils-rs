use color_eyre::eyre::{Result, WrapErr};
use rusoto_core::Region;
use rusoto_s3::S3Client;
use structopt::StructOpt;

use aws_logs_utils::log_types::FlowLogLine;
use aws_logs_utils::Parser;

#[derive(Debug, StructOpt)]
pub struct Options {
    pub region: String,
    pub bucket: String,
    pub prefix: String,
    pub filter_query: String,
}

lazy_static::lazy_static! {
    pub (crate) static ref OPTIONS: Options = Options::from_args();
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::try_init().wrap_err("Error initializing log")?;
    let s3_client = S3Client::new(Region::default());
    let parser = Parser::new(
        &s3_client,
        &OPTIONS.bucket,
        &OPTIONS.prefix,
        &OPTIONS.filter_query,
    );
    parser.parse_logs::<FlowLogLine>()?;
    Ok(())
}
