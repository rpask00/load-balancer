use crate::load_balancer::strategy::least_connection::LeastConnectionStrategy;
use crate::load_balancer::strategy::round_robin::RoundRobinStrategy;
use crate::load_balancer::strategy::{LoadBalancerStrategy, LoadBalancingStrategy};
use crate::load_balancer::worker::Worker;
use axum::http::{Request, Uri};
use color_eyre::eyre::{eyre, Result};
use hyper::body::Incoming;
use std::str::FromStr;
use std::sync::Arc;

pub struct LoadBalancer {
    pub workers: Vec<Arc<Worker>>,
    pub strategy: Box<dyn LoadBalancingStrategy>,
    free_ports: Vec<u16>,
    next_port: u16,
}

impl LoadBalancer {
    pub fn new(strategy: Box<dyn LoadBalancingStrategy>) -> Result<Self> {
        Ok(LoadBalancer {
            workers: vec![],
            strategy,
            free_ports: vec![],
            next_port: 3000,
        })
    }

    fn next_port(&self) -> u16 {
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

    pub fn spawn_worker(&mut self, num_threads: u8, name: String, port: Option<u16>) {
        let port = port.unwrap_or_else(|| self.next_port());
        self.workers
            .push(Arc::new(Worker::new(name, port, num_threads)));
    }

    pub fn close_worker(&mut self, worker_index: usize) {
        let worker = self.workers.remove(worker_index);
        std::thread::spawn(async move || {
            worker.shutdown().await.expect("Failed to shutdown worker");
        });
    }

    fn strategy_from_name(name: &str) -> Result<Box<dyn LoadBalancingStrategy>> {
        match LoadBalancerStrategy::from_str(name)
            .map_err(|_| eyre!("Unknown strategy name: {}", name))?
        {
            LoadBalancerStrategy::LeastConnections => Ok(Box::new(LeastConnectionStrategy::new())),
            LoadBalancerStrategy::RoundRobin => Ok(Box::new(RoundRobinStrategy::new())),
        }
    }

    pub fn set_strategy_handler(&mut self, strategy_name: &str) -> Result<()> {
        self.strategy = LoadBalancer::strategy_from_name(strategy_name)?;
        Ok(())
    }
}
