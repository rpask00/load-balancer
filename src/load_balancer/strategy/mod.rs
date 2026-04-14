use color_eyre::eyre::Result;
use std::sync::Arc;
use strum::EnumString;
use crate::load_balancer::worker::Worker;

pub trait LoadBalancingStrategy: Send + Sync {
     fn new() -> Self
    where
        Self: Sized;
    fn select_worker(&self, workers: &Vec<Arc<Worker>>) -> Result<Arc<Worker>>;
}

#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum LoadBalancerStrategy {
    RoundRobin,
    LeastConnections,
}

pub mod least_connection;
pub mod round_robin;
