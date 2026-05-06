use super::{ComponentAction, HandleEvent};
use crate::load_balancer::load_balancer::LoadBalancer;
use crate::tui::models::InputField;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Position, Rect},
    widgets::TableState,
};

pub struct AddItemMenu {
    pub name: String,
    pub port_str: String,
    pub focused: InputField,
    pub port_error: bool,
    pub popup_area: Option<Rect>,
    pub name_input_area: Option<Rect>,
    pub port_input_area: Option<Rect>,
}

impl Default for AddItemMenu {
    fn default() -> Self {
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
}

impl AddItemMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(&mut self, load_balancer: &mut LoadBalancer, table_state: &mut TableState) {
        let port = self.port_str.parse::<u16>();
        self.port_error = port.is_err();

        if let Ok(port) = port {
            if self.name.is_empty() {
                return;
            };

            if load_balancer
                .spawn_worker(1, self.name.clone(), Some(port))
                .is_ok()
            {
                table_state.select(Some(load_balancer.workers.len() - 1));
            }
        }
    }
}

impl HandleEvent for AddItemMenu {
    fn handle_key(&mut self, key: KeyEvent) -> ComponentAction {
        match key.code {
            KeyCode::Esc => ComponentAction::Cancel,
            KeyCode::Enter => ComponentAction::Submit,
            KeyCode::Tab => {
                self.focused = match self.focused {
                    InputField::Name => InputField::Port,
                    InputField::Port => InputField::Name,
                };
                ComponentAction::Continue
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
                ComponentAction::Continue
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
                ComponentAction::Continue
            }
            _ => ComponentAction::Continue,
        }
    }

    fn handle_mouse(&mut self, pos: Position) -> ComponentAction {
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
                ComponentAction::Continue
            } else {
                ComponentAction::Cancel
            }
        } else {
            ComponentAction::Continue
        }
    }
}
