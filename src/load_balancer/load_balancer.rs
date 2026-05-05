use crate::config::{FIRST_WORKER_PORT, MAX_WORKERS_COUNT};
use crate::load_balancer::strategy::least_connection::LeastConnectionStrategy;
use crate::load_balancer::strategy::round_robin::RoundRobinStrategy;
use crate::load_balancer::strategy::{LoadBalancerStrategy, LoadBalancingStrategy};
use crate::load_balancer::worker::Worker;
use axum::http::{Request, Uri};
use color_eyre::eyre::eyre;
use hyper::body::Incoming;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::Arc;

pub struct LoadBalancer {
    pub workers: Vec<Arc<Worker>>,
    pub strategy: Box<dyn LoadBalancingStrategy>,
    ports_pool: VecDeque<u16>,
}

impl LoadBalancer {
    pub fn new(strategy: Box<dyn LoadBalancingStrategy>) -> color_eyre::Result<Self> {
        Ok(LoadBalancer {
            workers: vec![],
            strategy,
            ports_pool: (FIRST_WORKER_PORT..FIRST_WORKER_PORT + MAX_WORKERS_COUNT).collect(),
        })
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
            .body(req.into_body())?;

        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        Ok(new_req)
    }

    pub fn spawn_worker(
        &mut self,
        num_threads: u8,
        name: String,
        port: Option<u16>,
    ) -> color_eyre::Result<()> {
        let port = port
            .or_else(|| self.ports_pool.pop_front())
            .ok_or_else(|| eyre!("No available ports to spawn new worker"))?;

        let worker = Worker::new(name, port, num_threads)?;
        self.workers.push(Arc::new(worker));

        Ok(())
    }

    pub fn close_worker(&mut self, worker_index: usize) {
        if worker_index < self.workers.len() {
            let worker = self.workers[worker_index].clone();
            let _ = worker.close();
        }
    }

    pub fn health_check(&self) {
        for worker in &self.workers {
            worker.health_check();
        }
    }

    fn strategy_from_name(name: &str) -> color_eyre::Result<Box<dyn LoadBalancingStrategy>> {
        match LoadBalancerStrategy::from_str(name)
            .map_err(|_| eyre!("Unknown strategy name: {}", name))?
        {
            LoadBalancerStrategy::LeastConnections => Ok(Box::new(LeastConnectionStrategy::new())),
            LoadBalancerStrategy::RoundRobin => Ok(Box::new(RoundRobinStrategy::new())),
        }
    }

    pub fn set_strategy_handler(&mut self, strategy_name: &str) -> color_eyre::Result<()> {
        self.strategy = LoadBalancer::strategy_from_name(strategy_name)?;
        Ok(())
    }

    pub async fn prune_workers(&mut self) {
        let closed_workers = self.workers.extract_if(.., |worker| {
            !worker.is_running() && Arc::strong_count(worker) == 1
        });

        for worker in closed_workers {
            let _ = worker.shutdown().await;
            self.ports_pool.push_back(worker.port);
        }
    }
}
