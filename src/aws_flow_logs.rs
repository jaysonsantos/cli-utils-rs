use color_eyre::eyre::{Result, WrapErr};
use lazy_static::lazy_static;
use tokio::runtime::{Builder, Runtime};

use crate::aws_logs_utils::{parse_logs, OPTIONS};
use crate::log_types::FlowLogLine;

mod aws_logs_utils;
mod aws_s3_utils;
mod log_types;

lazy_static! {
    pub static ref RUNTIME: Runtime = Builder::new()
        .enable_all()
        .threaded_scheduler() // This is needed otherwise Handle::block_on will hang
        .build()
        .expect("failed to create runtime");
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::try_init().wrap_err("Error initializing log")?;
    parse_logs::<FlowLogLine>(&OPTIONS)?;
    Ok(())
}
