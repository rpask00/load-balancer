use ratatui::{layout::Rect, widgets::TableState};

use crate::tui::models::{InputField, Item};

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
}
