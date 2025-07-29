use anyhow::Context;
use ungrammar::Grammar;

use crate::{
    codegen::grammar::{
        ast_node::generate_ast_node, ast_token::generate_ast_token, lower,
        syntax_kind::generate_syntax_kind,
    },
    utils::{ensure_rustfmt, project_root_path},
};

pub fn run() -> Result<(), anyhow::Error> {
    let grammar = std::fs::read_to_string(project_root_path().join("toml.ungram"))
        .unwrap()
        .parse::<Grammar>()
        .unwrap();

    let ast = lower(&grammar);

    ensure_rustfmt()?;

    write_file(
        &generate_syntax_kind()
            .context("Failed to generate syntax kind from grammar.".to_string())?,
        &project_root_path().join("crates/syntax/src/generated/syntax_kind.rs"),
    );

    write_file(
        &generate_ast_node(&ast)
            .context("Failed to generate ast node from grammar.".to_string())?,
        &project_root_path().join("crates/ast/src/generated/ast_node.rs"),
    );

    write_file(
        &generate_ast_token().context("Failed to generate ast node from grammar.".to_string())?,
        &project_root_path().join("crates/ast/src/generated/ast_token.rs"),
    );

    Ok(())
}

#[cfg(not(test))]
fn write_file(content: &str, target: &std::path::Path) {
    if !target.exists() {
        std::fs::create_dir_all(target.parent().unwrap())
            .unwrap_or_else(|err| panic!("Failed to create directory: {err}"));
    }
    std::fs::write(target, content)
        .unwrap_or_else(|err| panic!("Failed to write file {}: {}", target.display(), err));
}

#[cfg(test)]
fn write_file(_content: &str, _target: &std::path::Path) {}
