use std::env::args;
use std::fmt::Debug;

use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use http::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use log::trace;

struct Client {
    client: reqwest::Client,
    base_url: Url,
}

impl Client {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: Self::build_client()?,
            base_url: base_url.parse()?,
        })
    }

    fn build_client() -> Result<reqwest::Client> {
        let mut map = HeaderMap::new();
        map.insert(
            CONTENT_TYPE,
            "application/vnd.ksql.v1+json; charset=utf-8".parse()?,
        );

        Ok(reqwest::ClientBuilder::new().default_headers(map).build()?)
    }

    pub async fn query<T>(&self, statement: &str) -> Result<T>
    where
        T: DeserializeOwned + Debug,
    {
        println!("Running {:?}", statement);
        let response = self
            .client
            .post(self.base_url.clone())
            .json(&Query::with_statement(statement))
            .send()
            .await?;

        let json_response = response.json().await.wrap_err("failed to run query");
        trace!("Query result {:#?}", json_response);
        Ok(json_response?)
    }
}

#[derive(Debug, Serialize)]
struct Query {
    ksql: String,
}

impl Query {
    pub fn with_statement(statement: &str) -> Self {
        Self {
            ksql: statement.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Warnings {
    warnings: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct Message {
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::try_init()?;

    let topics_regex = regex::Regex::new(r"\[(\w+)")?;
    let queries_regex = regex::Regex::new(r"CSAS_\w+")?;

    let client = Client::new(args().nth(1).expect("specify the ksql url").as_str())?;
    let response = client
        .query::<Vec<Warnings>>("show streams extended;")
        .await?;
    let broken_streams: Vec<&str> = response
        .iter()
        .map(|warning| {
            warning.warnings.iter().map(|message| {
                topics_regex
                    .captures(&message.message)
                    .and_then(|captures| captures.get(1))
                    .map(|capture| capture.as_str())
                    .unwrap()
            })
        })
        .flatten()
        .collect();
    for query in broken_streams
        .iter()
        .map(|stream| format!("drop stream {};", stream))
    {
        'inner: loop {
            let response = client.query::<Value>(&query).await?;
            if response.is_array() {
                let command_status = response
                    .as_array()
                    .and_then(|r| r.get(0))
                    .and_then(|r| r.get("commandStatus"));

                let option = &command_status
                    .and_then(|s| s.get("status"))
                    .and_then(|s| s.as_str());
                let message = &command_status
                    .and_then(|s| s.get("message"))
                    .and_then(|m| m.as_str());
                println!(
                    "Query finished with status: {:?} message: {:?}",
                    option, message
                );
                break 'inner;
            }
            let query_to_kill = response
                .get("message")
                .and_then(|m| m.as_str())
                .and_then(|message| queries_regex.find(message))
                .map(|m| m.as_str())
                .unwrap();
            println!(
                "There are queries still running, killing {:?} before trying to drop the stream",
                query_to_kill
            );
            let _response: Value = client
                .query(&format!("terminate {};", query_to_kill))
                .await?;
        }
    }
    Ok(())
}
