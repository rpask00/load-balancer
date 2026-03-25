use crate::worker::Worker;

pub trait LoadBalancingStrategy: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn select_worker<'a>(&'a mut self, workers: &'a mut Vec<Worker>) -> &'a mut Worker;
}

pub mod least_connection;
pub mod round_robin;
