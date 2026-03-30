mod strategy;
mod worker;
use crate::strategy::least_connection::LeastConnectionStrategy;
use crate::strategy::round_robin::RoundRobinStrategy;
use crate::strategy::{LoadBalancerStrategy, LoadBalancingStrategy};
use crate::worker::Worker;
use bytes::Bytes;
use color_eyre::eyre::{eyre, Result};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::server::conn::http1;
use hyper::{body::Incoming, service::service_fn, Method, Request, Response, Uri};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use tokio::{net::TcpListener, task};

type BodyError = Box<dyn std::error::Error + Send + Sync>;
type BoxBodyResponse = Response<BoxBody<Bytes, BodyError>>;

struct LoadBalancer {
    workers: Vec<Arc<Worker>>,
    strategy: Box<dyn LoadBalancingStrategy>,
}

impl LoadBalancer {
    pub fn new(
        worker_hosts: Vec<String>,
        strategy: Box<dyn LoadBalancingStrategy>,
    ) -> Result<Self> {
        if worker_hosts.is_empty() {
            return Err(eyre!("No worker hosts provided"));
        }

        Ok(LoadBalancer {
            workers: worker_hosts
                .into_iter()
                .map(|url| Arc::new(Worker::new(url)))
                .collect(),
            strategy,
        })
    }

    pub async fn forward_request(&self, req: Request<Incoming>) -> Result<BoxBodyResponse> {
        let worker = self.strategy.select_worker(&self.workers)?;

        let mut worker_uri = worker.url.to_owned();

        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        let new_uri = Uri::from_str(&worker_uri)?;

        let headers = req.headers().clone();

        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .body(req.into_body())
            .expect("request builder");

        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        worker
            .handle(new_req)
            .await
            .map(|res| res.map(|body| body.map_err(|e| e.into()).boxed()))
            .map_err(|e| e.into())
    }
}

async fn handle(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<BoxBodyResponse> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/strategy") => set_strategy_handler(req, load_balancer).await,
        _ => load_balancer.read().await.forward_request(req).await,
    }
}

async fn set_strategy_handler(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<BoxBodyResponse> {
    let body = req.collect().await?.to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();

    let strategy_name = body["strategy"]
        .as_str()
        .ok_or(eyre!("Strategy not found in request body!"))?;

    let strategy = strategy_from_name(strategy_name);

    let (status, response_msg) = match strategy.is_ok() {
        true => (200, "ok"),
        false => (400, "unknown strategy"),
    };

    if let Ok(strategy) = strategy {
        load_balancer.write().await.strategy = strategy;
    }

    Ok(Response::builder().status(status).body(
        Full::new(Bytes::from(response_msg))
            .map_err(|_| unreachable!())
            .boxed(),
    )?)
}

fn strategy_from_name(name: &str) -> Result<Box<dyn LoadBalancingStrategy>> {
    match LoadBalancerStrategy::from_str(name)? {
        LoadBalancerStrategy::RoundRobin => Ok(Box::new(LeastConnectionStrategy::new())),
        LoadBalancerStrategy::LeastConnections => Ok(Box::new(RoundRobinStrategy::new())),
    }
}

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
        "http://localhost:3002".to_string(),
    ];

    let default_strategy = Box::new(LeastConnectionStrategy::new());
    // let default_strategy = Box::new(RoundRobinStrategy::new());

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
