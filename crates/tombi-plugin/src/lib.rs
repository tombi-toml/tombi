pub trait TombiPlugin {
    fn goto_definition(&self, path: &std::path::Path) -> Result<String, String>;
}
