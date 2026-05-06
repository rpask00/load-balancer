use crate::load_balancer::load_balancer::LoadBalancer;
use crate::load_balancer::strategy::LoadBalancerStrategy;
use crate::tui::component::{
    add_item_menu::AddItemMenu, main_menu::MainMenu, mode_select_menu::ModeSelectMenu,
    ComponentAction, HandleEvent,
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    layout::{Position, Rect},
    widgets::TableState,
};
use std::sync::{Arc, RwLock};

pub struct App {
    // app state vars
    pub table_state: TableState,
    pub load_balancer: Arc<RwLock<LoadBalancer>>,
    pub current_mode: LoadBalancerStrategy,
    pub should_quit: bool,
    // sub-components
    pub main_menu: MainMenu,
    pub add_item_menu: Option<AddItemMenu>,
    pub options_menu: Option<Rect>,
    pub mode_selector_menu: Option<ModeSelectMenu>,
}

impl App {
    pub fn new(load_balancer: Arc<RwLock<LoadBalancer>>) -> Self {
        Self {
            table_state: TableState::default().with_selected(Some(0)),
            current_mode: LoadBalancerStrategy::RoundRobin,
            should_quit: false,
            load_balancer,
            main_menu: MainMenu::new(),
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
        let mut load_balancer = self
            .load_balancer
            .write()
            .expect("Failed to lock load balancer for writing");

        if let Some(menu) = &mut self.add_item_menu {
            menu.submit(&mut load_balancer, &mut self.table_state);
            drop(load_balancer);

            if !menu.port_error {
                self.cancel_adding();
            }
        }
    }

    pub fn delete_at(&mut self, index: usize) {
        let mut load_balancer = self
            .load_balancer
            .write()
            .expect("Failed to lock load balancer for writing");

        load_balancer.close_worker(index);
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
        self.mode_selector_menu = Some(ModeSelectMenu::new(&self.current_mode));
    }

    pub fn confirm_mode_selection(&mut self) {
        if let Some(menu) = &mut self.mode_selector_menu {
            menu.confirm(&mut self.current_mode);
        }

        let mut load_balancer = self
            .load_balancer
            .write()
            .expect("Failed to lock load balancer for writing");



        load_balancer
            .set_strategy_handler((&self.current_mode).into())
            .expect("Failed to set load balancer strategy");

        self.mode_selector_menu = None;
    }

    pub fn cancel_mode_selection(&mut self) {
        self.mode_selector_menu = None;
    }

    pub fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_event(key),
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_event(mouse)
            }
            _ => false,
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if self.mode_selector_menu.is_some() {
            self.handle_key_mode_selector(key);
            return true;
        }
        if self.options_menu.is_some() {
            self.handle_key_options_menu(key);
            return true;
        }
        if self.add_item_menu.is_some() {
            self.handle_key_add_menu(key);
            return true;
        }

        let action = self.main_menu.handle_key(key);
        self.apply_action(action);
        true
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> bool {
        let pos = Position::new(mouse.column, mouse.row);

        if self.mode_selector_menu.is_some() {
            self.handle_mouse_mode_selector(pos);
            return true;
        }
        if self.add_item_menu.is_some() {
            self.handle_mouse_add_menu(pos);
            return true;
        }
        if self.options_menu.is_some() {
            self.handle_mouse_options_menu(pos);
            return true;
        }

        let action = self.main_menu.handle_mouse(pos);
        self.apply_action(action);
        true
    }

    fn apply_action(&mut self, action: ComponentAction) {
        match action {
            ComponentAction::Quit => self.should_quit = true,
            ComponentAction::DeleteSelected => self.delete_selected(),
            ComponentAction::StartAdding => self.start_adding(),
            ComponentAction::ToggleOptions => self.toggle_options_menu(),
            ComponentAction::TableSelectNext => self.table_state.select_next(),
            ComponentAction::TableSelectPrevious => self.table_state.select_previous(),
            ComponentAction::SelectTableRow(row_idx) => self.table_state.select(Some(row_idx)),
            _ => {}
        }
    }

    fn handle_key_mode_selector(&mut self, key: KeyEvent) {
        if let Some(menu) = &mut self.mode_selector_menu {
            let action = menu.handle_key(key);
            match action {
                ComponentAction::Cancel => self.cancel_mode_selection(),
                ComponentAction::Confirm => self.confirm_mode_selection(),
                _ => {}
            }
        }
    }

    fn handle_key_add_menu(&mut self, key: KeyEvent) {
        if let Some(menu) = &mut self.add_item_menu {
            let action = menu.handle_key(key);
            match action {
                ComponentAction::Cancel => self.cancel_adding(),
                ComponentAction::Submit => self.submit_adding(),
                _ => {}
            }
        }
    }

    fn handle_mouse_mode_selector(&mut self, pos: Position) {
        if let Some(menu) = &mut self.mode_selector_menu {
            let action = menu.handle_mouse(pos);
            match action {
                ComponentAction::Cancel => self.cancel_mode_selection(),
                ComponentAction::Confirm => self.confirm_mode_selection(),
                _ => {}
            }
        }
    }

    fn handle_mouse_add_menu(&mut self, pos: Position) {
        if let Some(menu) = &mut self.add_item_menu {
            let action = menu.handle_mouse(pos);
            match action {
                ComponentAction::Cancel => self.cancel_adding(),
                ComponentAction::Submit => self.submit_adding(),
                _ => {}
            }
        }
    }

    fn handle_key_options_menu(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.options_menu = None;
        }
    }

    fn handle_mouse_options_menu(&mut self, pos: Position) {
        if let Some(area) = self.options_menu {
            if area.contains(pos) {
                let rel_y = pos.y.saturating_sub(area.y + 1);
                if rel_y == 0 {
                    self.options_menu = None;
                    self.open_mode_select();
                } else if rel_y == 1 {
                    self.should_quit = true;
                }
            } else {
                self.options_menu = None;
            }
        }
    }
}
