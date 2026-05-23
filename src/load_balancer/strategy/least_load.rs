use crate::load_balancer::strategy::LoadBalancingStrategy;
use crate::load_balancer::worker::Worker;
use color_eyre::eyre::{eyre, Result};
use std::sync::Arc;

pub struct LeastLoadStrategy {}

impl LoadBalancingStrategy for LeastLoadStrategy {
    fn new() -> Self {
        LeastLoadStrategy {}
    }

    fn select_worker(&self, workers: &[Arc<Worker>]) -> Result<Arc<Worker>> {
        if workers.is_empty() {
            return Err(eyre!("There are no workers to select form!"));
        }

        Ok(workers
            .iter()
            .filter(|w| w.is_running())
            .reduce(|a, b| {
                let a_load = Arc::strong_count(a) as f64 / a.num_threads as f64;
                let b_load = Arc::strong_count(b) as f64 / b.num_threads as f64;
                if b_load > a_load {
                    a
                } else {
                    b
                }
            })
            .ok_or(eyre!("Failed to select worker"))?
            .clone())
    }
}
