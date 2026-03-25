use hyper::body::Incoming;
use hyper::{Request, Response};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::{Client, Error};
use hyper_util::rt::TokioExecutor;

pub struct Worker {
    client: Client<HttpConnector, Incoming>,
    pub url: String,
    pub connections_count: u32,
}

impl Worker {
    pub fn new(url: String) -> Self {
        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        Worker {
            url,
            client,
            connections_count: 0,
        }
    }

    pub async fn handle(&mut self, req: Request<Incoming>) -> Result<Response<Incoming>, Error> {
        self.connections_count += 1;
        let result = self.client.request(req).await;
        self.connections_count -= 1;
        result
    }
}
