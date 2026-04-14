use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Position, Rect},
    widgets::TableState,
};

use crate::tui::models::{InputField, Item};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AddMenuAction {
    Continue,
    Cancel,
    Submit,
}

pub struct AddItemMenu {
    pub name: String,
    pub port_str: String,
    pub focused: InputField,
    pub port_error: bool,
    pub popup_area: Option<Rect>,
    pub name_input_area: Option<Rect>,
    pub port_input_area: Option<Rect>,
}

impl AddItemMenu {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            port_str: String::new(),
            focused: InputField::Name,
            port_error: false,
            popup_area: None,
            name_input_area: None,
            port_input_area: None,
        }
    }

    pub fn submit(&mut self, items: &mut Vec<Item>, table_state: &mut TableState) {
        self.port_error = false;
        if let Ok(port) = self.port_str.parse::<u16>() {
            if !self.name.is_empty() {
                items.push(Item {
                    name: self.name.clone(),
                    port,
                });
                table_state.select(Some(items.len() - 1));
                return;
            }
        }
        self.port_error = true;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AddMenuAction {
        match key.code {
            KeyCode::Esc => AddMenuAction::Cancel,
            KeyCode::Enter => AddMenuAction::Submit,
            KeyCode::Tab => {
                self.focused = match self.focused {
                    InputField::Name => InputField::Port,
                    InputField::Port => InputField::Name,
                };
                AddMenuAction::Continue
            }
            KeyCode::Backspace => {
                match self.focused {
                    InputField::Name => {
                        let _ = self.name.pop();
                    }
                    InputField::Port => {
                        let _ = self.port_str.pop();
                        self.port_error = false;
                    }
                }
                AddMenuAction::Continue
            }
            KeyCode::Char(c) => {
                match self.focused {
                    InputField::Name => self.name.push(c),
                    InputField::Port if c.is_numeric() => {
                        self.port_str.push(c);
                        self.port_error = false;
                    }
                    _ => {}
                }
                AddMenuAction::Continue
            }
            _ => AddMenuAction::Continue,
        }
    }

    pub fn handle_mouse(&mut self, pos: Position) -> AddMenuAction {
        if let (Some(popup_area), Some(name_area), Some(port_area)) =
            (self.popup_area, self.name_input_area, self.port_input_area)
        {
            if popup_area.contains(pos) {
                if name_area.contains(pos) {
                    self.focused = InputField::Name;
                } else if port_area.contains(pos) {
                    self.focused = InputField::Port;
                    self.port_error = false;
                }
                AddMenuAction::Continue
            } else {
                AddMenuAction::Cancel
            }
        } else {
            AddMenuAction::Continue
        }
    }
}
