use crate::load_balancer::decision_engine::{DecisionEngine, Rule};
use crate::load_balancer::strategy::LoadBalancingPolicy;

pub struct Engine1 {
    rules: Vec<Box<Rule>>,
}

impl Default for Engine1 {
    fn default() -> Self {
        Self {
            rules: vec![Box::new(|lb_lock| {
                if lb_lock.workers.len() <= 5 {
                    Some(LoadBalancingPolicy::LeastConnections)
                } else {
                    Some(LoadBalancingPolicy::RoundRobin)
                }
            })],
        }
    }
}

impl DecisionEngine for Engine1 {
    fn rules(&self) -> &[Box<Rule>] {
        &self.rules
    }
}
