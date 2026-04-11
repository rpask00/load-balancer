use std::io::Write;
use crate::BoxBodyResponse;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::Request;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::{Client, Error};
use hyper_util::rt::TokioExecutor;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};

pub struct Worker {
    client: Client<HttpConnector, Incoming>,
    pub port: u32,
    child: Arc<RwLock<Child>>,
}

impl Worker {
    pub fn new(port: u32) -> Self {
        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        let child = Command::new("./target/debug/worker")
            .arg(port.to_string())
            .stdin(Stdio::piped()) // open a pipe to child's stdin
            .spawn()
            .unwrap();

        Worker { port, client, child: Arc::new(RwLock::new(child)), }
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

impl Drop for Worker {
    fn drop(&mut self) {
        let mut child = self.child.write().unwrap();
        child.stdin.as_mut().unwrap().write_all(b"shutdown\n").unwrap();
        child.wait().unwrap();
    }
}
