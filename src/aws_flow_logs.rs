use color_eyre::eyre::{Result, WrapErr};

use crate::aws_logs_utils::{parse_logs, OPTIONS};
use crate::log_types::FlowLogLine;

mod aws_logs_utils;
mod aws_s3_utils;
mod log_types;

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::try_init().wrap_err("Error initializing log")?;
    parse_logs::<FlowLogLine>(&OPTIONS)?;
    Ok(())
}
