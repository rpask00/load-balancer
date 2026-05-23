use crate::load_balancer::strategy::least_connection::LeastConnectionStrategy;
use crate::load_balancer::strategy::least_load::LeastLoadStrategy;
use crate::load_balancer::strategy::round_robin::RoundRobinStrategy;
use crate::load_balancer::worker::Worker;
use color_eyre::eyre::Result;
use std::sync::Arc;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

pub trait LoadBalancingStrategy: Send + Sync {
    fn policy(&self) -> LoadBalancingPolicy;
    fn new() -> Self
    where
        Self: Sized;
    fn select_worker(&self, workers: &[Arc<Worker>]) -> Result<Arc<Worker>>;
}

#[derive(Display, EnumString, EnumIter, IntoStaticStr, Clone, PartialEq)]
pub enum LoadBalancingPolicy {
    #[strum(serialize = "Round Robin")]
    RoundRobin,
    #[strum(serialize = "Least Connections")]
    LeastConnections,
    #[strum(serialize = "Least Load")]
    LeastLoad,
}

type StrategyConstructor = Box<dyn Fn() -> Box<dyn LoadBalancingStrategy> + Send + Sync>;

pub struct StrategyRegistry {
    entries: Vec<(LoadBalancingPolicy, StrategyConstructor)>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn register(
        &mut self,
        policy: LoadBalancingPolicy,
        ctor: impl Fn() -> Box<dyn LoadBalancingStrategy> + Send + Sync + 'static,
    ) {
        self.entries.push((policy, Box::new(ctor)));
    }

    pub fn build(&self, policy: &LoadBalancingPolicy) -> Option<Box<dyn LoadBalancingStrategy>> {
        self.entries
            .iter()
            .find(|(p, _)| p == policy)
            .map(|(_, ctor)| ctor())
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        for v in LoadBalancingPolicy::iter() {
            registry.register(v.clone(), move || match v {
                LoadBalancingPolicy::RoundRobin => Box::new(RoundRobinStrategy::new()),
                LoadBalancingPolicy::LeastConnections => Box::new(LeastConnectionStrategy::new()),
                LoadBalancingPolicy::LeastLoad => Box::new(LeastLoadStrategy::new()),
            });
        }

        registry
    }
}

pub mod least_connection;
pub mod least_load;
pub mod round_robin;
