use std::sync::Arc;
use crate::worker::Worker;

pub trait LoadBalancingStrategy: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn select_worker(&self, workers: &Vec<Arc<Worker>>) -> Arc<Worker>;
}

pub mod least_connection;
pub mod round_robin;
