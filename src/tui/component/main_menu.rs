use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Position, Rect};

use super::{ComponentAction, HandleEvent};

#[derive(Default)]
pub struct MainMenu {
    pub add_button_area: Option<Rect>,
    pub delete_button_area: Option<Rect>,
    pub options_button_area: Option<Rect>,
    pub table_area: Option<Rect>,
}

impl MainMenu {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HandleEvent for MainMenu {
    fn handle_key(&mut self, key: KeyEvent) -> ComponentAction {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => ComponentAction::Quit,
            KeyCode::Down | KeyCode::Char('j') => ComponentAction::TableSelectNext,
            KeyCode::Up | KeyCode::Char('k') => ComponentAction::TableSelectPrevious,
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('x') | KeyCode::Char('X') => {
                ComponentAction::DeleteSelected
            }
            KeyCode::Char('a') | KeyCode::Char('A') => ComponentAction::StartAdding,
            _ => ComponentAction::Continue,
        }
    }

    fn handle_mouse(&mut self, pos: Position) -> ComponentAction {
        if let Some(area) = self.add_button_area {
            if area.contains(pos) {
                return ComponentAction::StartAdding;
            }
        }
        if let Some(area) = self.delete_button_area {
            if area.contains(pos) {
                return ComponentAction::DeleteSelected;
            }
        }
        if let Some(area) = self.options_button_area {
            if area.contains(pos) {
                return ComponentAction::ToggleOptions;
            }
        }
        if let Some(table_area) = self.table_area {
            if table_area.contains(pos) {
                let relative_y = pos.y.saturating_sub(table_area.top());
                const DATA_START: u16 = 3;
                if relative_y >= DATA_START {
                    let row_idx = (relative_y - DATA_START) as usize;
                    return ComponentAction::SelectTableRow(row_idx);
                }
            }
        }
        ComponentAction::Continue
    }
}
