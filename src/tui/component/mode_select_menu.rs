use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Position, Rect};
use crate::load_balancer::strategy::LoadBalancerStrategy;
use super::{ComponentAction, HandleEvent};

pub struct ModeSelectMenu {
    pub selection_index: usize,
    pub menu_area: Option<Rect>,
}

impl ModeSelectMenu {
    pub fn new(current_mode: &LoadBalancerStrategy) -> Self {
        let selection_index = match current_mode {
            LoadBalancerStrategy::RoundRobin => 0,
            LoadBalancerStrategy::LeastConnections => 1,
        };
        Self {
            selection_index,
            menu_area: None,
        }
    }

    pub fn confirm(&mut self, current_mode: &mut LoadBalancerStrategy) {
        *current_mode = if self.selection_index == 0 {
            LoadBalancerStrategy::RoundRobin
        } else {
            LoadBalancerStrategy::LeastConnections
        };
    }
}

impl HandleEvent for ModeSelectMenu {
    fn handle_key(&mut self, key: KeyEvent) -> ComponentAction {
        match key.code {
            KeyCode::Esc => ComponentAction::Cancel,
            KeyCode::Enter => ComponentAction::Confirm,
            KeyCode::Down | KeyCode::Char('j') => {
                self.selection_index = (self.selection_index + 1) % 2;
                ComponentAction::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selection_index = if self.selection_index == 0 { 1 } else { 0 };
                ComponentAction::Continue
            }
            _ => ComponentAction::Continue,
        }
    }

    fn handle_mouse(&mut self, pos: Position) -> ComponentAction {
        if let Some(area) = self.menu_area {
            if area.contains(pos) {
                let relative_y = pos.y.saturating_sub(area.y + 4);
                if relative_y == 2 {
                    self.selection_index = 0;
                    return ComponentAction::Confirm;
                } else if relative_y == 5 {
                    self.selection_index = 1;
                    return ComponentAction::Confirm;
                }
            } else {
                return ComponentAction::Cancel;
            }
        }
        ComponentAction::Continue
    }
}
