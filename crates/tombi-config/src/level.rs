#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigLevel {
    Project,
    User,
    System,
    Default,
}
