use crate::tui::models::*;
use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{
    layout::{Position, Rect},
    widgets::TableState,
};

pub struct App {
    pub items: Vec<Item>,
    pub table_state: TableState,
    pub add_button_area: Option<Rect>,
    pub delete_button_area: Option<Rect>,
    pub options_button_area: Option<Rect>,
    pub options_menu_open: bool,
    pub options_menu_area: Option<Rect>,
    pub table_area: Option<Rect>,
    pub mode: AppMode,
    pub new_name: String,
    pub new_port_str: String,
    pub focused_field: InputField,
    pub should_quit: bool,
    pub current_mode: LoadBalancerMode,
    pub mode_select_open: bool,
    pub mode_select_area: Option<Rect>,
    pub mode_selection_index: usize,
    pub add_popup_area: Option<Rect>,
    pub add_name_input_area: Option<Rect>,
    pub add_port_input_area: Option<Rect>,
    pub add_port_error: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            items: vec![
                Item {
                    name: "worker 1".to_string(),
                    port: 3000,
                },
                Item {
                    name: "worker 2".to_string(),
                    port: 3001,
                },
                Item {
                    name: "worker 3".to_string(),
                    port: 3002,
                },
            ],
            table_state: TableState::default().with_selected(Some(0)),
            add_button_area: None,
            delete_button_area: None,
            options_button_area: None,
            options_menu_open: false,
            options_menu_area: None,
            table_area: None,
            mode: AppMode::Normal,
            new_name: String::new(),
            new_port_str: String::new(),
            focused_field: InputField::Name,
            should_quit: false,
            current_mode: LoadBalancerMode::RoundRobin,
            mode_select_open: false,
            mode_select_area: None,
            mode_selection_index: 0,
            add_popup_area: None,
            add_name_input_area: None,
            add_port_input_area: None,
            add_port_error: false,
        }
    }

    pub fn start_adding(&mut self) {
        self.mode = AppMode::Adding;
        self.new_name.clear();
        self.new_port_str.clear();
        self.focused_field = InputField::Name;
        self.add_port_error = false;
    }

    pub fn cancel_adding(&mut self) {
        self.mode = AppMode::Normal;
        self.new_name.clear();
        self.new_port_str.clear();
        self.add_port_error = false;
    }

    pub fn submit_adding(&mut self) {
        self.add_port_error = false;
        if let Ok(port) = self.new_port_str.parse::<u16>() {
            if !self.new_name.is_empty() {
                self.items.push(Item {
                    name: self.new_name.clone(),
                    port,
                });
                self.table_state.select(Some(self.items.len() - 1));
                self.cancel_adding();
                return;
            }
        }
        self.add_port_error = true;
    }

    pub fn delete_at(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            if !self.items.is_empty() {
                let new_idx = index
                    .saturating_sub(1)
                    .min(self.items.len().saturating_sub(1));
                self.table_state.select(Some(new_idx));
            } else {
                self.table_state.select(None);
            }
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(i) = self.table_state.selected() {
            self.delete_at(i);
        }
    }

    pub fn toggle_options_menu(&mut self) {
        self.options_menu_open = !self.options_menu_open;
    }

    pub fn open_mode_select(&mut self) {
        self.mode_selection_index = match self.current_mode {
            LoadBalancerMode::RoundRobin => 0,
            LoadBalancerMode::LeastConnections => 1,
        };
        self.mode_select_open = true;
    }

    pub fn confirm_mode_selection(&mut self) {
        self.current_mode = if self.mode_selection_index == 0 {
            LoadBalancerMode::RoundRobin
        } else {
            LoadBalancerMode::LeastConnections
        };
        self.mode_select_open = false;
        self.mode_select_area = None;
    }

    pub fn cancel_mode_selection(&mut self) {
        self.mode_select_open = false;
        self.mode_select_area = None;
    }

    pub fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if self.mode_select_open {
                    match key.code {
                        KeyCode::Esc => self.cancel_mode_selection(),
                        KeyCode::Enter => self.confirm_mode_selection(),
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.mode_selection_index = (self.mode_selection_index + 1) % 2
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.mode_selection_index =
                                if self.mode_selection_index == 0 { 1 } else { 0 }
                        }
                        _ => {}
                    }
                    return false;
                }

                if self.options_menu_open && key.code == KeyCode::Esc {
                    self.options_menu_open = false;
                    self.options_menu_area = None;
                    return false;
                }

                match self.mode {
                    AppMode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => self.table_state.select_next(),
                        KeyCode::Up | KeyCode::Char('k') => self.table_state.select_previous(),
                        KeyCode::Char('d')
                        | KeyCode::Char('D')
                        | KeyCode::Char('x')
                        | KeyCode::Char('X') => self.delete_selected(),
                        KeyCode::Char('a') | KeyCode::Char('A') => self.start_adding(),
                        _ => {}
                    },
                    AppMode::Adding => match key.code {
                        KeyCode::Esc => self.cancel_adding(),
                        KeyCode::Enter => self.submit_adding(),
                        KeyCode::Tab => {
                            self.focused_field = match self.focused_field {
                                InputField::Name => InputField::Port,
                                InputField::Port => InputField::Name,
                            };
                        }
                        KeyCode::Backspace => match self.focused_field {
                            InputField::Name => {
                                let _ = self.new_name.pop();
                            }
                            InputField::Port => {
                                let _ = self.new_port_str.pop();
                                self.add_port_error = false;
                            }
                        },
                        KeyCode::Char(c) => match self.focused_field {
                            InputField::Name => self.new_name.push(c),
                            InputField::Port if c.is_numeric() => {
                                self.new_port_str.push(c);
                                self.add_port_error = false;
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                }
            }
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let pos = Position::new(mouse.column, mouse.row);

                    if self.mode_select_open {
                        if let Some(area) = self.mode_select_area {
                            if area.contains(pos) {
                                let relative_y = mouse.row.saturating_sub(area.y + 4);
                                if relative_y == 2 {
                                    self.mode_selection_index = 0;
                                    self.confirm_mode_selection();
                                } else if relative_y == 5 {
                                    self.mode_selection_index = 1;
                                    self.confirm_mode_selection();
                                }
                                return false;
                            } else {
                                self.cancel_mode_selection();
                                return false;
                            }
                        }
                    }

                    if self.mode == AppMode::Adding {
                        if let (Some(popup_area), Some(name_area), Some(port_area)) = (
                            self.add_popup_area,
                            self.add_name_input_area,
                            self.add_port_input_area,
                        ) {
                            if popup_area.contains(pos) {
                                if name_area.contains(pos) {
                                    self.focused_field = InputField::Name;
                                } else if port_area.contains(pos) {
                                    self.focused_field = InputField::Port;
                                    self.add_port_error = false;
                                }
                                return false;
                            } else {
                                self.cancel_adding();
                                return false;
                            }
                        }
                    }

                    if self.options_menu_open {
                        if let Some(menu_area) = self.options_menu_area {
                            if menu_area.contains(pos) {
                                let rel_y = mouse.row.saturating_sub(menu_area.y + 1);
                                if rel_y == 0 {
                                    self.options_menu_open = false;
                                    self.options_menu_area = None;
                                    self.open_mode_select();
                                    return false;
                                } else if rel_y == 1 {
                                    self.should_quit = true;
                                    return false;
                                }
                                self.options_menu_open = false;
                                self.options_menu_area = None;
                                return false;
                            } else {
                                self.options_menu_open = false;
                                self.options_menu_area = None;
                                return false;
                            }
                        }
                    }

                    if let Some(area) = self.add_button_area {
                        if area.contains(pos) && self.mode == AppMode::Normal {
                            self.start_adding();
                            return false;
                        }
                    }
                    if let Some(area) = self.delete_button_area {
                        if area.contains(pos) {
                            self.delete_selected();
                            return false;
                        }
                    }
                    if let Some(area) = self.options_button_area {
                        if area.contains(pos) {
                            self.toggle_options_menu();
                            return false;
                        }
                    }

                    if self.mode == AppMode::Normal {
                        if let Some(table_area) = self.table_area {
                            if table_area.contains(pos) {
                                let relative_y = mouse.row.saturating_sub(table_area.top());
                                const DATA_START: u16 = 3;
                                if relative_y >= DATA_START {
                                    let row_idx = (relative_y - DATA_START) as usize;
                                    if row_idx < self.items.len() {
                                        self.table_state.select(Some(row_idx));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }
}
