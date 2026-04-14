pub mod add_item_menu;
pub mod main_menu;
pub mod mode_select_menu;

pub trait HandleEvent {
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> ComponentAction;
    fn handle_mouse(&mut self, pos: ratatui::layout::Position) -> ComponentAction;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComponentAction {
    Continue,
    Cancel,
    Submit,
    Confirm,
    Quit,
    DeleteSelected,
    StartAdding,
    ToggleOptions,
    SelectTableRow(usize),
    TableSelectNext,
    TableSelectPrevious,
}
