use crate::load_balancer::strategy::least_connection::LeastConnectionStrategy;
use crate::load_balancer::strategy::round_robin::RoundRobinStrategy;
use crate::load_balancer::worker::Worker;
use color_eyre::eyre::Result;
use std::sync::Arc;
use strum::{Display, EnumString, IntoStaticStr};

pub trait LoadBalancingStrategy: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn select_worker(&self, workers: &[Arc<Worker>]) -> Result<Arc<Worker>>;
}

#[derive(Display, EnumString, IntoStaticStr, Clone)]
pub enum LoadBalancingPolicy {
    #[strum(serialize = "Round Robin")]
    RoundRobin,
    #[strum(serialize = "Least Connections")]
    LeastConnections,
}

impl LoadBalancingPolicy {
    pub fn build(self) -> Box<dyn LoadBalancingStrategy> {
        match self {
            LoadBalancingPolicy::LeastConnections => Box::new(LeastConnectionStrategy::new()),
            LoadBalancingPolicy::RoundRobin => Box::new(RoundRobinStrategy::new()),
        }
    }
}

pub mod least_connection;
pub mod round_robin;
