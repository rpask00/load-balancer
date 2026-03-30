use crate::strategy::LoadBalancingStrategy;
use crate::worker::Worker;
use color_eyre::eyre::{eyre, Result};
use std::sync::{Arc, Mutex};

pub struct RoundRobinStrategy {
    current_worker_index: Mutex<usize>,
}

impl LoadBalancingStrategy for RoundRobinStrategy {
    fn new() -> Self {
        RoundRobinStrategy {
            current_worker_index: Mutex::new(0),
        }
    }

    fn select_worker(&self, workers: &Vec<Arc<Worker>>) -> Result<Arc<Worker>> {
        println!("{}", "Round Robin is selecting worker");

        if workers.len() == 0 {
            return Err(eyre!("There are no workers to select form!"));
        }

        let mut current_worker_index_lock = self
            .current_worker_index
            .lock()
            .map_err(|e| eyre!(e.to_string()))?;

        *current_worker_index_lock = (*current_worker_index_lock + 1) % workers.len();
        let current_worker = &workers[*current_worker_index_lock];

        Ok(Arc::clone(current_worker))
    }
}
