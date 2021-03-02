/// TODO: Disclaimer, many parts of this code will still block so some refactoring is needed
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::Read;

use color_eyre::eyre::{Result, WrapErr};
use color_eyre::Report;
use flate2::read::MultiGzDecoder;
use futures::executor::block_on;
use log::{debug, info, trace};
use rusoto_s3::{S3Client, S3};
use serde::de::DeserializeOwned;
use wirefilter::FilterAst;

use aws_s3_utils::BucketKeyIterator;

use crate::log_types::Searchable;

pub mod log_types;

lazy_static::lazy_static! {
    static ref INTERESTING_ERRORS: HashSet<u16> = (501..=504).collect();
    static ref IGNORED_ELB_ERRORS: HashSet<u16> = (300..=460).collect();
}

pub struct Parser<'a> {
    client: &'a S3Client,
    bucket: &'a str,
    prefix: &'a str,
    filter_query: &'static str,
}

impl<'a> Parser<'a> {
    pub fn new(
        client: &'a S3Client,
        bucket: &'a str,
        prefix: &'a str,
        filter_query: &'static str,
    ) -> Self {
        Self {
            client,
            bucket,
            prefix,
            filter_query,
        }
    }

    pub fn parse_logs<S>(&self) -> Result<()>
    where
        S: Searchable + DeserializeOwned + Debug,
    {
        debug!("Starting process");
        let scheme = S::scheme();
        let ast = scheme
            .parse(self.filter_query)
            .wrap_err("failed to parse filter query")?;
        let mut iterator = BucketKeyIterator::new(self.bucket, Some(self.prefix), self.client);

        while let Some(key) = block_on(iterator.iter_next())? {
            self.process_log_file::<S>(&key, &ast)?;
        }

        Ok(())
    }

    fn process_log_file<'ast, S>(&self, key: &str, ast: &'ast FilterAst<'a>) -> Result<()>
    where
        S: Searchable + DeserializeOwned + Debug,
    {
        let data = self.process_s3_file::<S>(key, true);
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
                Self::process_log_line(line, ast)?;
            }
        }

        Ok(())
    }

    fn process_s3_file<T>(
        &self,
        key: &str,
        has_headers: bool,
    ) -> Option<Result<impl Iterator<Item = std::result::Result<T, csv::Error>>>>
    where
        T: DeserializeOwned,
    {
        debug!("Starting to download from s3://{}/{}", self.bucket, key);
        let request = rusoto_s3::GetObjectRequest {
            bucket: self.bucket.to_owned(),
            key: key.to_owned(),
            ..Default::default()
        };
        let response = match block_on(self.client.get_object(request))
            .wrap_err("Error downloading log file")
        {
            Ok(response) => response,
            Err(e) => return Some(Err(e)),
        };

        if let Some(body) = response.body {
            debug!(
                "Processing bucket: {} key: {} size: {}",
                self.bucket,
                key,
                human_format::Formatter::new()
                    .with_scales(human_format::Scales::Binary())
                    .with_units("B")
                    .format(response.content_length.unwrap() as f64)
            );
            let stream =
                Self::read_log_file(MultiGzDecoder::new(body.into_blocking_read()), has_headers);
            return Some(Ok(stream));
        } else {
            info!(
                "Nothing useful returned from s3 file bucket: {} key: {} response: {:?}",
                self.bucket, key, response
            )
        }

        None
    }

    fn read_log_file<T, R>(
        file: R,
        has_headers: bool,
    ) -> impl Iterator<Item = Result<T, csv::Error>>
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
}
