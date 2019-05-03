use std::collections::HashSet;
use std::io::Read;

use failure::ResultExt;
use flate2::read::MultiGzDecoder;

use log::debug;
use log::info;
use rusoto_s3::{S3Client, S3};
use serde::de::DeserializeOwned;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    pub region: String,
    pub bucket: String,
    pub prefix: String,
    pub filter_query: String,
}

lazy_static::lazy_static! {
    static ref INTERESTING_ERRORS: HashSet<u16> = (501..=504).into_iter().collect();
    static ref IGNORED_ELB_ERRORS: HashSet<u16> = (300..=460).into_iter().collect();
    pub (crate) static ref OPTIONS: Options = Options::from_args();
    pub (crate) static ref S3_CLIENT: S3Client = S3Client::new(OPTIONS.region.parse().unwrap());
}
//
pub(crate) fn process_s3_file<T>(
    bucket: &str,
    key: &str,
    has_headers: bool,
) -> Option<Result<impl Iterator<Item = Result<T, csv::Error>>, failure::Error>>
where
    T: DeserializeOwned,
{
    debug!("Starting to download from s3://{}/{}", bucket, key);
    let request = rusoto_s3::GetObjectRequest {
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        ..Default::default()
    };
    let response = match S3_CLIENT
        .get_object(request)
        .sync()
        .context("Error downloading log file")
    {
        Ok(response) => response,
        Err(e) => return Some(Err(e.into())),
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
        return Some(Ok(parse_log_file(
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

fn parse_log_file<T, R>(file: R, has_headers: bool) -> impl Iterator<Item = Result<T, csv::Error>>
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
