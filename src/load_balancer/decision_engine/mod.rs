pub mod engine1;
pub mod engine2;

use crate::load_balancer::load_balancer::LoadBalancer;
use crate::load_balancer::strategy::LoadBalancingPolicy;

pub trait DecisionEngine: Send + Sync {
    fn rules(&self) -> &[Box<Rule>];

    fn select_strategy(&self, load_balancer: &mut LoadBalancer) {
        let mut final_strategy = LoadBalancingPolicy::RoundRobin;
        for rule in self.rules() {
            if let Some(s) = rule(load_balancer) {
                final_strategy = s;
            }
        }

        let _ = load_balancer.set_strategy(final_strategy);
    }
}

pub type Rule = dyn Fn(&LoadBalancer) -> Option<LoadBalancingPolicy> + Send + Sync;
