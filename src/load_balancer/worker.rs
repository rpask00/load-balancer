use crate::load_balancer::BoxBodyResponse;
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
use strum::Display;
use tokio::task::spawn_blocking;

pub struct Worker {
    pub name: String,
    pub status: RwLock<WorkerStatus>,
    pub port: u16,
    client: Client<HttpConnector, Incoming>,
    pub num_threads: u8,
    child: Arc<RwLock<Child>>,
}

#[derive(Display, PartialEq, Copy, Clone)]
pub enum WorkerStatus {
    Running,
    Closing,
    Closed,
    Unknown,
}

impl Worker {
    pub fn new(name: String, port: u16, num_threads: u8) -> Self {
        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        let child = Command::new("./target/debug/lb_worker")
            .arg("--port")
            .arg(port.to_string())
            .arg("--num-threads")
            .arg(num_threads.to_string())
            .stdin(Stdio::piped()) // open a pipe to child's stdin
            .spawn()
            .unwrap();

        Worker {
            port,
            name,
            num_threads,
            client,
            status: RwLock::new(WorkerStatus::Running),
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

    pub fn is_running(&self) -> bool {
        if let Ok(status) = self.status.read() {
            return *status == WorkerStatus::Running;
        }

        false
    }

    pub fn close(&self) -> color_eyre::Result<()> {
        let mut status = self.status.write().map_err(|e| eyre!(e.to_string()))?;
        *status = WorkerStatus::Closing;

        Ok(())
    }

    pub async fn shutdown(&self) -> color_eyre::Result<()> {
        let child = self.child.clone();

        spawn_blocking::<_, color_eyre::Result<()>>(move || {
            let mut child = child.write().map_err(|e| eyre!(e.to_string()))?;
            child
                .stdin
                .as_mut()
                .ok_or(eyre!("Failed to open stdin for child process"))?
                .write_all(b"shutdown\n")?;

            child.wait()?;

            Ok(())
        })
        .await??;

        {
            let mut status = self.status.write().map_err(|e| eyre!(e.to_string()))?;
            *status = WorkerStatus::Closed;
        }

        Ok(())
    }
}
