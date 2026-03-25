use crate::strategy::LoadBalancingStrategy;
use crate::worker::Worker;

pub struct LeastConnectionStrategy {}

impl LoadBalancingStrategy for LeastConnectionStrategy {
    fn new() -> Self {
        LeastConnectionStrategy {}
    }

    fn select_worker<'a>(&'a mut self, workers: &'a mut Vec<Worker>) -> &'a mut Worker {
        workers
            .iter_mut()
            .reduce(|a, b| {
                if (b.connections_count > a.connections_count) {
                    b
                } else {
                    a
                }
            })
            .unwrap()
    }
}
