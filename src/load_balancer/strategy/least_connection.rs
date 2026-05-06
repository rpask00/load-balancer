use crate::load_balancer::strategy::LoadBalancingStrategy;
use crate::load_balancer::worker::Worker;
use color_eyre::eyre::{eyre, Result};
use std::sync::Arc;

pub struct LeastConnectionStrategy {}

impl LoadBalancingStrategy for LeastConnectionStrategy {
    fn new() -> Self {
        LeastConnectionStrategy {}
    }

    fn select_worker(&self, workers: &Vec<Arc<Worker>>) -> Result<Arc<Worker>> {
        if workers.is_empty() {
            return Err(eyre!("There are no workers to select form!"));
        }

        Ok(workers
            .iter()
            .filter(|w| w.is_running())
            .reduce(|a, b| {
                if Arc::strong_count(b) > Arc::strong_count(a) {
                    a
                } else {
                    b
                }
            })
            .ok_or(eyre!("Failed to select worker"))?
            .clone())
    }
}
