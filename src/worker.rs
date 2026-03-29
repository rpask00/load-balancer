use crate::BoxBodyResponse;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::Request;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::{Client, Error};
use hyper_util::rt::TokioExecutor;

pub struct Worker {
    client: Client<HttpConnector, Incoming>,
    pub url: String,
}

impl Worker {
    pub fn new(url: String) -> Self {
        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        Worker { url, client }
    }

    pub async fn handle(&self, req: Request<Incoming>) -> Result<BoxBodyResponse, Error> {
        let result = self
            .client
            .request(req)
            .await
            .map(|res| res.map(|body| body.map_err(|e| e.into()).boxed()));
        result
    }
}
