pub mod codegen;
pub mod codegen_grammar;
pub mod codegen_jsonschema;
pub mod dist;
pub mod set_version;
pub mod toml_test;

pub use codegen::CodeGenCommand;

#[derive(Debug, clap::Subcommand)]
pub enum XTaskCommand {
    /// Generate code.
    #[clap(subcommand)]
    Codegen(CodeGenCommand),

    /// Set Git Tag version.
    SetVersion,

    /// Run toml-test.
    TomlTest(toml_test::Args),

    /// Prepare the distribution.
    Dist,
}
