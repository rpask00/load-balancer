use crate::load_balancer::strategy::LoadBalancerStrategy;
use std::sync::{Arc, RwLock};

use crate::load_balancer::load_balancer::LoadBalancer;

type Rule = dyn Fn(&LoadBalancer) -> Option<LoadBalancerStrategy> + Send + Sync;

pub struct DecisionEngine {
    load_balancer: Arc<RwLock<LoadBalancer>>,
    rules: Vec<Box<Rule>>,
}

impl DecisionEngine {
    pub fn new(load_balancer: Arc<RwLock<LoadBalancer>>) -> Self {
        Self {
            load_balancer,
            rules: vec![Box::new(|lb_lock| {
                if lb_lock.workers.len() <= 5 {
                    Some(LoadBalancerStrategy::LeastConnections)
                } else {
                    Some(LoadBalancerStrategy::RoundRobin)
                }
            })],
        }
    }

    pub fn select_strategy(&mut self) -> LoadBalancerStrategy {
        let mut lb_lock = self.load_balancer.write().expect("");
        let mut final_strategy = LoadBalancerStrategy::RoundRobin;
        for rule in self.rules.iter() {
            if let Some(strategy) = rule(&*lb_lock) {
                final_strategy = strategy;
            }
        }

        let _ = lb_lock.set_strategy_handler(&final_strategy.to_string());

        final_strategy
    }
}
