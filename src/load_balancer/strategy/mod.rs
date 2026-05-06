use crate::load_balancer::worker::Worker;
use color_eyre::eyre::Result;
use strum::{Display, EnumString, IntoStaticStr};
use std::sync::Arc;

pub trait LoadBalancingStrategy: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn select_worker(&self, workers: &Vec<Arc<Worker>>) -> Result<Arc<Worker>>;
}

#[derive(Display, EnumString, IntoStaticStr, Clone)]
pub enum LoadBalancerStrategy {
    #[strum(serialize = "Round Robin")]
    RoundRobin,
    #[strum(serialize = "Least Connections")]
    LeastConnections,
}

pub mod least_connection;
pub mod round_robin;
