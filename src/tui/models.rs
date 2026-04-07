#[derive(Clone)]
pub struct Item {
    pub name: String,
    pub port: u16,
}

#[derive(PartialEq)]
pub enum AppMode {
    Normal,
    Adding,
}

#[derive(PartialEq, Clone, Copy)]
pub enum InputField {
    Name,
    Port,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LoadBalancerMode {
    RoundRobin,
    LeastConnections,
}

impl LoadBalancerMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoadBalancerMode::RoundRobin => "Round Robin",
            LoadBalancerMode::LeastConnections => "Least Connections",
        }
    }
}
