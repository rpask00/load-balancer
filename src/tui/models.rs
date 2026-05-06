#[derive(Clone)]
pub struct Item {
    pub name: String,
    pub port: u16,
}

#[derive(PartialEq)]
pub enum AppMode {
    Normal,
    Adding,
}

#[derive(PartialEq, Clone, Copy)]
pub enum InputField {
    Name,
    Port,
}
