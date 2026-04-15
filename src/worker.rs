use crate::BoxBodyResponse;
use color_eyre::eyre::eyre;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::Request;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::{Client, Error};
use hyper_util::rt::TokioExecutor;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};

pub struct Worker {
    pub port: u16,
    client: Client<HttpConnector, Incoming>,
    num_threads: u8,
    child: Arc<RwLock<Child>>,
}

impl Worker {
    pub fn new(port: u16, num_threads: u8) -> Self {
        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        let child = Command::new("./target/debug/worker")
            .arg("--port")
            .arg(port.to_string())
            .arg("--num-threads")
            .arg(num_threads.to_string())
            .stdin(Stdio::piped()) // open a pipe to child's stdin
            .spawn()
            .unwrap();

        Worker {
            port,
            num_threads,
            client,
            child: Arc::new(RwLock::new(child)),
        }
    }

    pub async fn handle(&self, req: Request<Incoming>) -> Result<BoxBodyResponse, Error> {
        let result = self
            .client
            .request(req)
            .await
            .map(|res| res.map(|body| body.map_err(|e| e.into()).boxed()));
        result
    }

    pub async fn shutdown(&self) -> color_eyre::Result<()> {
        let mut child = self
            .child
            .write()
            .map_err(|_| eyre!("Failed to acquire write lock on child process"))?;

        child
            .stdin
            .as_mut()
            .ok_or(eyre!("Failed to open stdin for child process"))?
            .write_all(b"shutdown\n")?;

        child.wait()?;

        Ok(())
    }
}
