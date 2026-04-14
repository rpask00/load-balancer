use ratatui::layout::Rect;

use crate::tui::models::LoadBalancerMode;

pub struct ModeSelectorMenu {
    pub selection_index: usize,
    pub menu_area: Option<Rect>,
}

impl ModeSelectorMenu {
    pub fn new(current_mode: LoadBalancerMode) -> Self {
        let selection_index = match current_mode {
            LoadBalancerMode::RoundRobin => 0,
            LoadBalancerMode::LeastConnections => 1,
        };
        Self {
            selection_index,
            menu_area: None,
        }
    }

    pub fn confirm(&mut self, current_mode: &mut LoadBalancerMode) {
        *current_mode = if self.selection_index == 0 {
            LoadBalancerMode::RoundRobin
        } else {
            LoadBalancerMode::LeastConnections
        };
    }
}
