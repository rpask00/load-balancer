use crate::load_balancer::strategy::least_connection::LeastConnectionStrategy;
use crate::load_balancer::strategy::round_robin::RoundRobinStrategy;
use crate::load_balancer::strategy::{LoadBalancerStrategy, LoadBalancingStrategy};
use crate::load_balancer::worker::Worker;
use axum::http::{Request, Uri};
use color_eyre::eyre::Result;
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

    fn next_port(&mut self) -> u16 {
        if self.free_ports.is_empty() {
            self.free_ports.push(self.next_port);
            self.next_port += 1;
        }

        // free_ports can't be empty at this point so it will never panic.
        self.free_ports.pop().expect("No free ports available")
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

    pub async fn close_worker(&mut self, worker_index: usize) -> Result<()> {
        let worker = self.workers.remove(worker_index);
        worker.shutdown().await?;
        self.free_ports.push(worker.port);

        Ok(())
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
