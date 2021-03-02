use rusoto_s3::{S3Client, S3};

use crate::error::{Error, Result};

mod error;

pub struct BucketKeyIterator<'a> {
    bucket: &'a str,
    prefix: Option<&'a str>,
    cli: &'a S3Client,
    continuation_token: Option<String>,
    keys: Vec<String>,
    empty: bool,
}

impl<'a> BucketKeyIterator<'a> {
    pub fn new(
        bucket: &'a str,
        prefix: Option<&'a str>,
        cli: &'a S3Client,
    ) -> BucketKeyIterator<'a> {
        BucketKeyIterator {
            bucket,
            prefix,
            cli,
            continuation_token: None,
            keys: vec![],
            empty: false,
        }
    }

    pub async fn iter_next(&mut self) -> Result<Option<String>> {
        // TODO: Use streams to implement the old iterator
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

        let response = self
            .cli
            .list_objects_v2(request)
            .await
            .map_err(|e| eyre::eyre!(e))?;
        if let Some(keys) = response.contents {
            let mut output_keys = vec![];
            for key in keys.iter().filter(|object| match object.key {
                Some(ref key) => {
                    key.ends_with(".gz") && self.prefix.map(|p| key.contains(p)).unwrap_or(false)
                }
                None => false,
            }) {
                let key = key.key.as_ref().ok_or(Error::KeyNotPresent)?;
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
