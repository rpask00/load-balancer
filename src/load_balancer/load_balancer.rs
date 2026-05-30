use crate::config::{FIRST_WORKER_PORT, MAX_WORKERS_COUNT};
use crate::load_balancer::strategy::{
    LoadBalancingPolicy, LoadBalancingStrategy, StrategyRegistry,
};
use crate::load_balancer::worker::Worker;
use axum::http::{Request, Uri};
use color_eyre::eyre::eyre;
use hyper::body::Incoming;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::sleep;

pub struct LoadBalancer {
    pub workers: Vec<Arc<Worker>>,
    pub strategy: Box<dyn LoadBalancingStrategy>,
    strategy_registry: StrategyRegistry,
    ports_pool: VecDeque<u16>,
}

impl LoadBalancer {
    pub fn new(strategy_policy: LoadBalancingPolicy) -> color_eyre::Result<Self> {
        let strategy_registry = StrategyRegistry::default();

        let strategy = strategy_registry
            .build(&strategy_policy)
            .ok_or_else(|| eyre!("Failed to initialize load balancing strategy!"))?;

        Ok(LoadBalancer {
            workers: vec![],
            strategy,
            strategy_registry,
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

    pub fn set_strategy(&mut self, strategy: LoadBalancingPolicy) -> color_eyre::Result<()> {
        self.strategy = self
            .strategy_registry
            .build(&strategy)
            .ok_or(eyre!("Failed to set strategy!"))?;
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

    pub async fn exit(&mut self) -> color_eyre::Result<()> {
        for worker in &mut self.workers {
            worker.close()?;
        }

        while !self.workers.is_empty() {
            self.prune_workers().await;
            sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }
}
