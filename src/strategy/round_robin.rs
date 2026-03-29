use std::sync::{Arc, Mutex};
use crate::strategy::LoadBalancingStrategy;
use crate::worker::Worker;

pub struct RoundRobinStrategy {
    current_worker_index: Mutex<usize>,
}

impl LoadBalancingStrategy for RoundRobinStrategy {
     fn new() -> Self {
        RoundRobinStrategy {
            current_worker_index: Mutex::new(0),
        }
    }

    fn select_worker(&self, workers: &Vec<Arc<Worker>>) ->  Arc<Worker> {
        println!("{}", "Round Robin is selecting worker");

        let workers_len = workers.len();

        let mut current_worker_index_lock = self.current_worker_index.lock().unwrap();

        let current_worker = &workers[*current_worker_index_lock];
        *current_worker_index_lock = (*current_worker_index_lock + 1) % workers_len;

        Arc::clone(current_worker)
    }
}
