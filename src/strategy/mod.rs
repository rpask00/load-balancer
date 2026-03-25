use crate::worker::Worker;

pub trait LoadBalancingStrategy: Sized {
    fn new() -> Self;
    fn select_worker<'a>(&'a mut self, workers: &'a mut Vec<Worker>) -> &'a mut Worker;
}

pub mod round_robin;
pub mod least_connection;
 