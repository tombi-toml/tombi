pub mod codegen;
pub mod codegen_grammar;
pub mod codegen_jsonschema;
pub mod dist;
pub mod json_schema_test;
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

    /// Run JSON-Schema-Test-Suite (Definition A).
    JsonSchemaTest(json_schema_test::Args),

    /// Prepare the distribution.
    Dist,
}
