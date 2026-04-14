mod load_balancer;
mod strategy;
mod worker;

use crate::load_balancer::LoadBalancer;
use crate::strategy::least_connection::LeastConnectionStrategy;
use crate::strategy::LoadBalancingStrategy;
use bytes::Bytes;
use color_eyre::eyre::{eyre, Result};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::{body::Incoming, service::service_fn, Method, Request, Response};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tokio::{net::TcpListener, task};

type BodyError = Box<dyn std::error::Error + Send + Sync>;
type BoxBodyResponse = Response<BoxBody<Bytes, BodyError>>;

async fn handle(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<BoxBodyResponse> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/strategy") => set_strategy_handler(req, load_balancer).await,
        _ => {
            let (worker, req) = {
                let lb_lock = load_balancer.read().await;
                let worker = lb_lock.strategy.select_worker(&lb_lock.workers)?;
                let req =
                    lb_lock.prepare_request(format!("http://localhost:{}", worker.port), req)?;
                (worker, req)
            };

            worker
                .handle(req)
                .await
                .map(|res| res.map(|body| body.map_err(|e| e.into()).boxed()))
                .map_err(|e| e.into())
        }
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

    let result = load_balancer
        .write()
        .await
        .set_strategy_handler(strategy_name);

    let (status, response_msg) = match result.is_ok() {
        true => (200, "ok"),
        false => (400, "unknown strategy"),
    };

    Ok(Response::builder().status(status).body(
        Full::new(Bytes::from(response_msg))
            .map_err(|_| unreachable!())
            .boxed(),
    )?)
}

#[tokio::main]
async fn main() {
    let default_strategy = Box::new(LeastConnectionStrategy::new());
    // let default_strategy = Box::new(RoundRobinStrategy::new());

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(default_strategy).expect("failed to create load balancer"),
    ));

    load_balancer.write().await.spawn_worker(5);
    // load_balancer.write().await.spawn_worker(1);
    // load_balancer.write().await.spawn_worker(1);
    // load_balancer.write().await.spawn_worker(1);
    // load_balancer.write().await.spawn_worker(1);

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
