use color_eyre::eyre::eyre;
use color_eyre::Result;

use rusoto_s3::{S3Client, S3};

use crate::aws_logs_utils::S3_CLIENT;

pub struct BucketKeyIterator<'a> {
    bucket: &'a str,
    prefix: Option<&'a str>,
    cli: &'static S3Client,
    continuation_token: Option<String>,
    keys: Vec<String>,
    empty: bool,
}

impl<'a> BucketKeyIterator<'a> {
    pub fn new(bucket: &'a str, prefix: Option<&'a str>) -> BucketKeyIterator<'a> {
        BucketKeyIterator {
            bucket,
            prefix,
            cli: &S3_CLIENT,
            continuation_token: None,
            keys: vec![],
            empty: false,
        }
    }

    fn iter_next(&mut self) -> Result<Option<String>> {
        if let Some(key) = self.keys.pop() {
            return Ok(Some(key));
        }

        if self.empty {
            return Ok(None);
        }

        let request = rusoto_s3::ListObjectsV2Request {
            bucket: self.bucket.to_owned(),
            prefix: self.prefix.map(|p| p.to_owned()),
            continuation_token: self.continuation_token.clone(),
            ..Default::default()
        };

        let response = self.cli.list_objects_v2(request).sync()?;
        if let Some(keys) = response.contents {
            let mut output_keys = vec![];
            for key in keys.iter().filter(|object| match object.key {
                Some(ref key) => {
                    key.ends_with(".gz") && self.prefix.map(|p| key.contains(p)).unwrap_or(false)
                }
                None => false,
            }) {
                let key = key
                    .key
                    .as_ref()
                    .ok_or_else(|| eyre!("Key was not present"))?;
                output_keys.push(key.clone());
            }

            output_keys.reverse();
            self.keys = output_keys;
            self.continuation_token = response.next_continuation_token;
            self.empty = !response.is_truncated.unwrap();
            return Ok(self.keys.pop());
        }

        Ok(None)
    }
}

impl<'a> Iterator for BucketKeyIterator<'a> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter_next() {
            Ok(res) => res.map(Ok),
            Err(err) => Some(Err(err)),
        }
    }
}
