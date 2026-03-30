use crate::worker::Worker;
use color_eyre::eyre::{eyre, Result};
use color_eyre::Report;
use std::sync::Arc;

pub trait LoadBalancingStrategy: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn select_worker(&self, workers: &Vec<Arc<Worker>>) -> Result<Arc<Worker>>;
}

pub enum LoadBalancerStrategy {
    RoundRobin,
    LeastConnections,
}

impl std::str::FromStr for LoadBalancerStrategy {
    type Err = Report;
    fn from_str(s: &str) -> Result<Self> {

        match s {
            "round_robin" => Ok(Self::RoundRobin),
            "least_connections" => Ok(Self::LeastConnections),
            _ => Err(eyre!("Unknown load balancer strategy: {}", s)),
        }
    }
}

pub mod least_connection;
pub mod round_robin;
