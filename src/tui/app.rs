use crate::tui::{
    component::{add_item_menu::AddItemMenu, mode_select_menu::ModeSelectorMenu},
    models::*,
};
use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{
    layout::{Position, Rect},
    widgets::TableState,
};

pub struct App {
    // app state vars
    pub items: Vec<Item>,
    pub table_state: TableState,
    pub current_mode: LoadBalancerMode,
    pub should_quit: bool,

    // default app ui vars
    pub add_button_area: Option<Rect>,
    pub delete_button_area: Option<Rect>,
    pub options_button_area: Option<Rect>,
    pub table_area: Option<Rect>,

    // conditional app ui vars
    pub add_item_menu: Option<AddItemMenu>,
    pub options_menu: Option<Rect>,
    pub mode_selector_menu: Option<ModeSelectorMenu>,
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
            current_mode: LoadBalancerMode::RoundRobin,
            should_quit: false,

            add_button_area: None,
            delete_button_area: None,
            options_button_area: None,
            table_area: None,

            add_item_menu: None,
            options_menu: None,
            mode_selector_menu: None,
        }
    }

    pub fn start_adding(&mut self) {
        self.add_item_menu = Some(AddItemMenu::new());
    }

    pub fn cancel_adding(&mut self) {
        self.add_item_menu = None;
    }

    pub fn submit_adding(&mut self) {
        if let Some(menu) = &mut self.add_item_menu {
            menu.submit(&mut self.items, &mut self.table_state);
            if !menu.port_error {
                self.cancel_adding();
            }
        }
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
        if self.options_menu.is_some() {
            self.options_menu = None;
        } else {
            self.options_menu = Some(Rect::default());
        }
    }

    pub fn open_mode_select(&mut self) {
        self.mode_selector_menu = Some(ModeSelectorMenu::new(self.current_mode));
    }

    pub fn confirm_mode_selection(&mut self) {
        if let Some(menu) = &mut self.mode_selector_menu {
            menu.confirm(&mut self.current_mode);
        }
        self.mode_selector_menu = None;
    }

    pub fn cancel_mode_selection(&mut self) {
        self.mode_selector_menu = None;
    }

    pub fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if let Some(menu) = &mut self.mode_selector_menu {
                    match key.code {
                        KeyCode::Esc => self.cancel_mode_selection(),
                        KeyCode::Enter => self.confirm_mode_selection(),
                        KeyCode::Down | KeyCode::Char('j') => {
                            menu.selection_index = (menu.selection_index + 1) % 2
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            menu.selection_index = if menu.selection_index == 0 { 1 } else { 0 }
                        }
                        _ => {}
                    }
                    return false;
                }

                if let Some(_) = &mut self.options_menu {
                    if key.code == KeyCode::Esc {
                        self.options_menu = None;
                        return false;
                    }
                }

                if let Some(menu) = &mut self.add_item_menu {
                    match key.code {
                        KeyCode::Esc => self.cancel_adding(),
                        KeyCode::Enter => self.submit_adding(),
                        KeyCode::Tab => {
                            menu.focused = match menu.focused {
                                InputField::Name => InputField::Port,
                                InputField::Port => InputField::Name,
                            };
                        }
                        KeyCode::Backspace => match menu.focused {
                            InputField::Name => {
                                let _ = menu.name.pop();
                            }
                            InputField::Port => {
                                let _ = menu.port_str.pop();
                                menu.port_error = false;
                            }
                        },
                        KeyCode::Char(c) => match menu.focused {
                            InputField::Name => menu.name.push(c),
                            InputField::Port if c.is_numeric() => {
                                menu.port_str.push(c);
                                menu.port_error = false;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    return false;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Down | KeyCode::Char('j') => self.table_state.select_next(),
                    KeyCode::Up | KeyCode::Char('k') => self.table_state.select_previous(),
                    KeyCode::Char('d')
                    | KeyCode::Char('D')
                    | KeyCode::Char('x')
                    | KeyCode::Char('X') => self.delete_selected(),
                    KeyCode::Char('a') | KeyCode::Char('A') => self.start_adding(),
                    _ => {}
                }
            }
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let pos = Position::new(mouse.column, mouse.row);

                    if let Some(menu) = &mut self.mode_selector_menu {
                        if let Some(area) = menu.menu_area {
                            if area.contains(pos) {
                                let relative_y = mouse.row.saturating_sub(area.y + 4);
                                if relative_y == 2 {
                                    menu.selection_index = 0;
                                    self.confirm_mode_selection();
                                } else if relative_y == 5 {
                                    menu.selection_index = 1;
                                    self.confirm_mode_selection();
                                }
                                return false;
                            } else {
                                self.cancel_mode_selection();
                                return false;
                            }
                        }
                    }

                    if let Some(menu) = &mut self.add_item_menu {
                        if let (Some(popup_area), Some(name_area), Some(port_area)) =
                            (menu.popup_area, menu.name_input_area, menu.port_input_area)
                        {
                            if popup_area.contains(pos) {
                                if name_area.contains(pos) {
                                    menu.focused = InputField::Name;
                                } else if port_area.contains(pos) {
                                    menu.focused = InputField::Port;
                                    menu.port_error = false;
                                }
                                return false;
                            } else {
                                self.cancel_adding();
                                return false;
                            }
                        }
                    }

                    if let Some(menu) = &mut self.options_menu {
                        if menu.contains(pos) {
                            let rel_y = mouse.row.saturating_sub(menu.y + 1);
                            if rel_y == 0 {
                                self.options_menu = None;
                                self.open_mode_select();
                                return false;
                            } else if rel_y == 1 {
                                self.should_quit = true;
                                return false;
                            }
                        }
                        self.options_menu = None;
                        return false;
                    }

                    if let Some(area) = self.add_button_area {
                        if area.contains(pos) {
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
            _ => {}
        }
        false
    }
}
