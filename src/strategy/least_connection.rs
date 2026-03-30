use crate::strategy::LoadBalancingStrategy;
use crate::worker::Worker;
use color_eyre::eyre::{eyre, Result};
use std::sync::Arc;

pub struct LeastConnectionStrategy {}

impl LoadBalancingStrategy for LeastConnectionStrategy {
    fn new() -> Self {
        LeastConnectionStrategy {}
    }

    fn select_worker(&self, workers: &Vec<Arc<Worker>>) -> Result<Arc<Worker>> {
        println!("{}", "Least connection is selecting worker");

        if workers.len() == 0 {
            return Err(eyre!("There are no workers to select form!"));
        }

        Ok(workers
            .iter()
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
