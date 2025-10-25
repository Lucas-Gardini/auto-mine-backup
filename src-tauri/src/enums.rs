#[derive(Debug, Clone, PartialEq)]
pub enum WorldStatus {
    Stopped,
    MainMenu,
    Multiplayer(String),
    InWorld(String),
}
