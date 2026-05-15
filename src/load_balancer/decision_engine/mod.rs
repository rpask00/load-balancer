pub mod engine1;
pub mod engine2;

use crate::load_balancer::load_balancer::LoadBalancer;
use crate::load_balancer::strategy::LoadBalancerStrategy;

pub trait DecisionEngine: Send + Sync {
    fn rules(&self) -> &[Box<Rule>];

    fn select_strategy(&mut self, load_balancer: &mut LoadBalancer) -> LoadBalancerStrategy {
        let mut final_strategy = LoadBalancerStrategy::RoundRobin;
        for rule in self.rules() {
            if let Some(s) = rule(load_balancer) {
                final_strategy = s;
            }
        }
        let _ = load_balancer.set_strategy_handler(&final_strategy.to_string());
        final_strategy
    }
}


pub type Rule = dyn Fn(&LoadBalancer) -> Option<LoadBalancerStrategy> + Send + Sync;
