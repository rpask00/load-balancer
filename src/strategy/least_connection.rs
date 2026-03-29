use std::sync::Arc;
use crate::strategy::LoadBalancingStrategy;
use crate::worker::Worker;

pub struct LeastConnectionStrategy {}

impl LoadBalancingStrategy for LeastConnectionStrategy {
    fn new() -> Self {
        LeastConnectionStrategy {}
    }

    fn select_worker(&self, workers: &Vec<Arc<Worker>>) ->  Arc<Worker> {
         println!("{}", "Least connection is selecting worker");
        
        
        workers
            .iter()
            .reduce(|a, b| {
                if Arc::strong_count(b) > Arc::strong_count(a) {
                    b
                } else {
                    a
                }
            })
            .unwrap()
            .clone()
    }
}
