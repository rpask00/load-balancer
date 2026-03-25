mod strategy;
mod worker;
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use crate::strategy::round_robin::RoundRobinStrategy;
use crate::strategy::LoadBalancingStrategy;
use crate::worker::Worker;
use hyper::server::conn::http1;
use hyper::{body::Incoming, service::service_fn, Request, Response, Uri};
use hyper_util::client::legacy::{Error as ClientError, Error};
use hyper_util::rt::TokioIo;
use tokio::sync::RwLock;
use tokio::{net::TcpListener, task};

struct LoadBalancer {
    worker_hosts: Vec<Worker>,
    strategy: RoundRobinStrategy,
}

impl LoadBalancer {
    pub fn new(worker_hosts: Vec<String>, strategy: RoundRobinStrategy) -> Result<Self, String> {
        if worker_hosts.is_empty() {
            return Err("No worker hosts provided".into());
        }

        Ok(LoadBalancer {
            worker_hosts: worker_hosts
                .into_iter()
                .map(|url| Worker::new(url))
                .collect(),
            strategy,
        })
    }

    pub async fn forward_request(
        &mut self,
        req: Request<Incoming>,
    ) -> Result<Response<Incoming>, Error> {
        let worker = self.get_worker();

        let mut worker_uri = worker.url.to_owned();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).unwrap();

        // Extract the headers from the original request
        let headers = req.headers().clone();

        // Clone the original request's headers and method
        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .body(req.into_body())
            .expect("request builder");

        // Copy headers from the original request
        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        worker.handle(new_req).await
    }

    fn get_worker(&mut self) -> &mut Worker {
        self.strategy.select_worker(&mut self.worker_hosts)
    }
}

async fn handle(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Incoming>, ClientError> {
    load_balancer.write().await.forward_request(req).await
}

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
        "http://localhost:3002".to_string(),
    ];

    let default_strategy = RoundRobinStrategy::new();

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(worker_hosts, default_strategy).expect("failed to create load balancer"),
    ));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    println!("load balancer listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await.expect("failed to accept");
        let load_balancer = load_balancer.clone();

        task::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| handle(req, load_balancer.clone()));

            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("error: {}", e);
            }
        });
    }
}
