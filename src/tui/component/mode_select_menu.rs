use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Position, Rect};

use crate::tui::models::LoadBalancerMode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModeSelectAction {
    Continue,
    Cancel,
    Confirm,
}

pub struct ModeSelectMenu {
    pub selection_index: usize,
    pub menu_area: Option<Rect>,
}

impl ModeSelectMenu {
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

    pub fn handle_key(&mut self, key: KeyEvent) -> ModeSelectAction {
        match key.code {
            KeyCode::Esc => ModeSelectAction::Cancel,
            KeyCode::Enter => ModeSelectAction::Confirm,
            KeyCode::Down | KeyCode::Char('j') => {
                self.selection_index = (self.selection_index + 1) % 2;
                ModeSelectAction::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selection_index = if self.selection_index == 0 { 1 } else { 0 };
                ModeSelectAction::Continue
            }
            _ => ModeSelectAction::Continue,
        }
    }

    pub fn handle_mouse(&mut self, pos: Position) -> ModeSelectAction {
        if let Some(area) = self.menu_area {
            if area.contains(pos) {
                let relative_y = pos.y.saturating_sub(area.y + 4);
                if relative_y == 2 {
                    self.selection_index = 0;
                    return ModeSelectAction::Confirm;
                } else if relative_y == 5 {
                    self.selection_index = 1;
                    return ModeSelectAction::Confirm;
                }
            } else {
                return ModeSelectAction::Cancel;
            }
        }
        ModeSelectAction::Continue
    }
}
