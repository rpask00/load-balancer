use crate::load_balancer::worker::Worker;
use color_eyre::eyre::Result;
use std::str::FromStr;
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

impl FromStr for LoadBalancerStrategy {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Round Robin" => Ok(LoadBalancerStrategy::RoundRobin),
            "Least Connections" => Ok(LoadBalancerStrategy::LeastConnections),
            &_ => Err(()),
        }
    }
}

impl LoadBalancerStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoadBalancerStrategy::RoundRobin => "Round Robin",
            LoadBalancerStrategy::LeastConnections => "Least Connections",
        }
    }
}

pub mod least_connection;
pub mod round_robin;
