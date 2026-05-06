use crate::load_balancer::strategy::LoadBalancingStrategy;
use crate::load_balancer::worker::Worker;
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
        if workers.is_empty() {
            return Err(eyre!("There are no workers to select form!"));
        }


        let running: Vec<&Arc<Worker>> = workers.iter().filter(|w| w.is_running()).collect();

        if running.is_empty() {
            return Err(eyre!("No running workers available!"));
        }

        let mut current_worker_index = self
            .current_worker_index
            .lock()
            .map_err(|e| eyre!(e.to_string()))?;

        *current_worker_index = (*current_worker_index + 1) % running.len();

        Ok(Arc::clone(running[*current_worker_index]))
    }
}
