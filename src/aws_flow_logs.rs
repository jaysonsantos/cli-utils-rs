use failure::ResultExt;
use log::debug;

use log_types::FlowLogLine;

use crate::aws_logs_utils::{process_s3_file, OPTIONS};
use crate::aws_s3_utils::BucketKeyIterator;

mod aws_logs_utils;
mod aws_s3_utils;
mod log_types;

fn main() -> Result<(), failure::Error> {
    env_logger::try_init().context("Error initializing log")?;
    debug!("Starting process");

    let scheme = FlowLogLine::scheme();
    let ast = scheme.parse(&OPTIONS.filter_query)?;

    for key in BucketKeyIterator::new(OPTIONS.bucket.as_str(), Some(OPTIONS.prefix.as_str())) {
        let key = key?;
        let data = process_s3_file::<FlowLogLine>(OPTIONS.bucket.as_str(), key.as_str(), true);
        debug!("Processing {}", key);
        if let Some(lines) = data {
            for line in lines? {
                let line = line?;
                let ctx = line
                    .execution_context()
                    .context("error building execution context")?;
                let filter = ast.clone().compile();

                if filter.execute(&ctx)? {
                    println!("Matched with {:#?}", line);
                    return Ok(());
                } else {
                    debug!("NOT Matched with {:#?}", line);
                    ();
                }
            }
        }
    }

    Ok(())
}
