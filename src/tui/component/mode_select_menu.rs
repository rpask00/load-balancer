use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Position, Rect};
use crate::load_balancer::strategy::LoadBalancingPolicy;
use super::{ComponentAction, HandleEvent};

pub struct ModeSelectMenu {
    pub selection_index: usize,
    pub menu_area: Option<Rect>,
}

impl ModeSelectMenu {
    pub fn new(current_mode: &LoadBalancingPolicy) -> Self {
        let selection_index = match current_mode {
            LoadBalancingPolicy::RoundRobin => 0,
            LoadBalancingPolicy::LeastConnections => 1,
            LoadBalancingPolicy::LeastLoad => 2,
        };
        Self {
            selection_index,
            menu_area: None,
        }
    }

    pub fn confirm(&mut self, current_mode: &mut LoadBalancingPolicy) {
        *current_mode = match self.selection_index {
            0 => LoadBalancingPolicy::RoundRobin,
            1 => LoadBalancingPolicy::LeastConnections,
            _ => LoadBalancingPolicy::LeastLoad,
        };
    }
}

impl HandleEvent for ModeSelectMenu {
    fn handle_key(&mut self, key: KeyEvent) -> ComponentAction {
        match key.code {
            KeyCode::Esc => ComponentAction::Cancel,
            KeyCode::Enter => ComponentAction::Confirm,
            KeyCode::Down | KeyCode::Char('j') => {
                self.selection_index = (self.selection_index + 1) % 3;
                ComponentAction::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selection_index = if self.selection_index == 0 { 2 } else { self.selection_index - 1 };
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
                } else if relative_y == 8 {
                    self.selection_index = 2;
                    return ComponentAction::Confirm;
                }
            } else {
                return ComponentAction::Cancel;
            }
        }
        ComponentAction::Continue
    }
}
