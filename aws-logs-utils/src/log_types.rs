use std::net::IpAddr;

use color_eyre::{Report, Result};
use serde::Deserialize;
use wirefilter::{ExecutionContext, Scheme};

pub trait Searchable {
    fn scheme() -> &'static Scheme;
    fn execution_context(&self) -> Result<ExecutionContext>;
}

//#[derive(Debug, Deserialize, PartialEq)]
//#[serde(untagged)]
//pub(crate) enum OptionalNumber<'a, T>
//where
//    T: Deserialize<'a>,
//{
//    Duration(T),
//    Failure(String),
//}
//
//#[derive(Debug, PartialEq)]
//pub(crate) struct Request {
//    pub method: String,
//    pub path: String,
//    pub http_version: String,
//}
//
//impl From<&str> for Request {
//    fn from(data: &str) -> Self {
//        let mut parts = data.split(" ");
//        Request {
//            method: parts.next().unwrap().to_owned(),
//            path: parts.next().unwrap().to_owned(),
//            http_version: parts.next().unwrap().to_owned(),
//        }
//    }
//}
//
//#[derive(Debug, Deserialize)]
//pub(crate) struct RequestLogLine {
//    pub request_type: String,
//    pub timestamp: DateTime<Utc>,
//    pub elb_name: String,
//    pub client: String,
//    pub target: String,
//    pub request_processing_time: OptionalNumber<f64>,
//    pub target_processing_time: OptionalNumber<f64>,
//    pub response_processing_time: OptionalNumber<f64>,
//    pub elb_status_code: u16,
//    pub target_status_code: OptionalNumber<u16>,
//    pub received_bytes: u64,
//    pub sent_bytes: u64,
//    // method + url + http version
//    request: String,
//    pub user_agent: String,
//    pub ssl_cipher: String,
//    pub ssl_protocol: String,
//    pub target_group_arn: String,
//    pub trace_id: String,
//    pub domain_name: String,
//    pub chosen_cert_arn: String,
//    pub matched_rule_priority: String,
//    pub request_creation_time: DateTime<Utc>,
//    pub actions_executed: String,
//    pub error_reason: String,
//}
//
//impl RequestLogLine {
//    pub fn request(&self) -> Request {
//        Request::from(self.request.as_str())
//    }
//}

#[derive(Debug, Deserialize)]
pub struct FlowLogLine {
    pub version: String,
    #[serde(alias = "account-id")]
    pub account_id: String,
    #[serde(alias = "interface-id")]
    pub interface_id: String,
    pub srcaddr: IpAddr,
    pub dstaddr: IpAddr,
    pub srcport: i32,
    pub dstport: i32,
    pub protocol: String,
    pub packets: String,
    pub bytes: i32,

    pub start: i32,
    pub end: i32,
    pub action: String,
    #[serde(alias = "log-status")]
    pub log_status: String,
}

lazy_static::lazy_static! {
    pub static ref FLOW_SCHEME: Scheme = Scheme! {
        srcport: Int,
        srcaddr: Ip,

        dstport: Int,
        dstaddr: Ip,

        start: Int,
        end: Int,

        bytes: Int,
        action: Bytes,
        log_status: Bytes,
    };
}

impl Searchable for FlowLogLine {
    fn scheme() -> &'static Scheme {
        &*FLOW_SCHEME
    }

    fn execution_context(&self) -> Result<ExecutionContext> {
        let mut ctx = ExecutionContext::new(Self::scheme());
        ctx.set_field_value("srcport", self.srcport)
            .map_err(Report::msg)?;
        ctx.set_field_value("srcaddr", self.srcaddr)
            .map_err(Report::msg)?;

        ctx.set_field_value("dstport", self.dstport)
            .map_err(Report::msg)?;
        ctx.set_field_value("dstaddr", self.dstaddr)
            .map_err(Report::msg)?;

        ctx.set_field_value("start", self.start)
            .map_err(Report::msg)?;
        ctx.set_field_value("end", self.end).map_err(Report::msg)?;

        ctx.set_field_value("bytes", self.bytes)
            .map_err(Report::msg)?;
        ctx.set_field_value("action", self.action.as_str())
            .map_err(Report::msg)?;
        ctx.set_field_value("log_status", self.log_status.as_str())
            .map_err(Report::msg)?;

        Ok(ctx)
    }
}
