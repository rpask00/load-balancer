use crate::strategy::LoadBalancingStrategy;
use crate::worker::Worker;

pub struct RoundRobinStrategy {
    current_worker_index: usize,
}

impl LoadBalancingStrategy for RoundRobinStrategy {
     fn new() -> Self {
        RoundRobinStrategy {
            current_worker_index: 0,
        }
    }

    fn select_worker<'a>(&mut self, workers: &'a mut Vec<Worker>) -> &'a mut Worker {
        let workers_len = workers.len();
        let current_worker = &mut workers[self.current_worker_index];
        self.current_worker_index = (self.current_worker_index + 1) % workers_len;
        current_worker
    }
}
