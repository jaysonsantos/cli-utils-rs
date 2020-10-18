use std::collections::HashSet;
use std::fmt::Debug;
use std::io::Read;

use color_eyre::eyre::{Result, WrapErr};
use color_eyre::Report;
use flate2::read::MultiGzDecoder;
use log::{debug, info, trace};
use rusoto_s3::{S3Client, S3};
use serde::de::DeserializeOwned;
use structopt::StructOpt;
use tokio_compat_02::FutureExt;
use wirefilter::FilterAst;

use crate::aws_s3_utils::BucketKeyIterator;
use crate::log_types::Searchable;
use crate::RUNTIME;

#[derive(Debug, StructOpt)]
pub struct Options {
    pub region: String,
    pub bucket: String,
    pub prefix: String,
    pub filter_query: String,
}

lazy_static::lazy_static! {
    static ref INTERESTING_ERRORS: HashSet<u16> = (501..=504).collect();
    static ref IGNORED_ELB_ERRORS: HashSet<u16> = (300..=460).collect();
    pub (crate) static ref OPTIONS: Options = Options::from_args();
    pub (crate) static ref S3_CLIENT: S3Client = S3Client::new(OPTIONS.region.parse().unwrap());
}

pub(crate) fn process_s3_file<T>(
    bucket: &str,
    key: &str,
    has_headers: bool,
) -> Option<Result<impl Iterator<Item = std::result::Result<T, csv::Error>>>>
where
    T: DeserializeOwned,
{
    debug!("Starting to download from s3://{}/{}", bucket, key);
    let request = rusoto_s3::GetObjectRequest {
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        ..Default::default()
    };
    let response = match RUNTIME
        .block_on(S3_CLIENT.get_object(request).compat())
        .wrap_err("Error downloading log file")
    {
        Ok(response) => response,
        Err(e) => return Some(Err(e)),
    };

    if let Some(body) = response.body {
        debug!(
            "Processing bucket: {} key: {} size: {}",
            bucket,
            key,
            human_format::Formatter::new()
                .with_scales(human_format::Scales::Binary())
                .with_units("B")
                .format(response.content_length.unwrap() as f64)
        );
        return Some(Ok(read_log_file(
            MultiGzDecoder::new(body.into_blocking_read()),
            has_headers,
        )));
    } else {
        info!(
            "Nothing useful returned from s3 file bucket: {} key: {} response: {:?}",
            bucket, key, response
        )
    }

    None
}

fn read_log_file<T, R>(file: R, has_headers: bool) -> impl Iterator<Item = Result<T, csv::Error>>
where
    T: DeserializeOwned,
    R: Read,
{
    csv::ReaderBuilder::new()
        .delimiter(b' ')
        .has_headers(has_headers)
        .from_reader(file)
        .into_deserialize()
}

pub(crate) fn parse_logs<S>(options: &'static Options) -> Result<()>
where
    S: Searchable + DeserializeOwned + Debug,
{
    debug!("Starting process");
    let scheme = S::scheme();
    let ast = scheme
        .parse(&options.filter_query)
        .wrap_err("failed to parse filter query")?;
    for key in BucketKeyIterator::new(options.bucket.as_str(), Some(options.prefix.as_str())) {
        let key = key?;
        process_log_file::<S>(options.bucket.as_str(), key.as_str(), &ast)?;
    }

    Ok(())
}

fn process_log_file<S>(bucket: &str, key: &str, ast: &FilterAst) -> Result<()>
where
    S: Searchable + DeserializeOwned + Debug,
{
    let data = process_s3_file::<S>(bucket, key, true);
    debug!("Processing {}", key);
    if let Some(lines) = data {
        for line in lines? {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    debug!("Skipping line because of {:?}", e);
                    continue;
                }
            };
            process_log_line(line, ast)?;
        }
    }

    Ok(())
}

fn process_log_line<S>(line: S, ast: &FilterAst) -> Result<()>
where
    S: Searchable + DeserializeOwned + Debug,
{
    let filter = ast.clone().compile();
    let ctx = line
        .execution_context()
        .wrap_err("error building execution context")?;
    if filter.execute(&ctx).map_err(Report::msg)? {
        println!("Matched with {:#?}", line);
    } else {
        trace!("NOT Matched with {:#?}", line);
    }
    Ok(())
}
