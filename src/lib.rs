use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Response;

pub mod strategy;
pub mod tui;
pub mod worker;

pub type BodyError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxBodyResponse = Response<BoxBody<Bytes, BodyError>>;
