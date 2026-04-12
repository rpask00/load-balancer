use crate::strategy::least_connection::LeastConnectionStrategy;
use crate::strategy::round_robin::RoundRobinStrategy;
use crate::strategy::{LoadBalancerStrategy, LoadBalancingStrategy};
use crate::worker::Worker;
use axum::http::{Request, Uri};
use hyper::body::Incoming;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task;

pub struct LoadBalancer {
    pub workers: Vec<Arc<Worker>>,
    pub strategy: Box<dyn LoadBalancingStrategy>,
}

impl LoadBalancer {
    pub fn new(strategy: Box<dyn LoadBalancingStrategy>) -> color_eyre::Result<Self> {
        Ok(LoadBalancer {
            workers: vec![],
            strategy,
        })
    }

    fn next_port(&self) -> u32 {
        let mut port = 3000;
        let mut used_ports = self.workers.iter().map(|w| w.port).collect::<Vec<_>>();
        used_ports.sort();

        for used_port in used_ports {
            if used_port != port {
                return port;
            };
            port += 1;
        }

        port
    }

    pub fn prepare_request(
        &self,
        mut worker_uri: String,
        req: Request<Incoming>,
    ) -> color_eyre::Result<Request<Incoming>> {
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        let new_uri = Uri::from_str(&worker_uri)?;

        let headers = req.headers().clone();

        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .body(req.into_body())
            .expect("Failed to build request");

        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        Ok(new_req)
    }

    pub fn spawn_worker(&mut self, num_threads: u8) {
        let port = self.next_port();
        self.workers.push(Arc::new(Worker::new(port, num_threads)));
    }

    pub async fn close_worker(&mut self, worker_index: usize) {
        let worker = self.workers.remove(worker_index);

        let _ = task::spawn_blocking(move || {
            drop(worker);
        })
        .await;
    }

    fn strategy_from_name(name: &str) -> color_eyre::Result<Box<dyn LoadBalancingStrategy>> {
        match LoadBalancerStrategy::from_str(name)? {
            LoadBalancerStrategy::RoundRobin => Ok(Box::new(LeastConnectionStrategy::new())),
            LoadBalancerStrategy::LeastConnections => Ok(Box::new(RoundRobinStrategy::new())),
        }
    }

    pub fn set_strategy_handler(&mut self, strategy_name: &str) -> color_eyre::Result<()> {
        self.strategy = LoadBalancer::strategy_from_name(&strategy_name)?;
        Ok(())
    }
}
