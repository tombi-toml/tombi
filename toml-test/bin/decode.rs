use std::io::Read;

use clap::Parser;
use tombi_ast::AstNode;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_toml_version::TomlVersion;
use toml_test::{IntoValue, Value, INVALID_MESSAGE};

#[derive(Debug, clap::Parser, Default)]
#[command(disable_help_subcommand(true))]
pub struct Args {
    #[clap(long)]
    pub toml_version: TomlVersion,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse_from(std::env::args_os());
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source)?;

    let value = decode(&source, args.toml_version)?;
    println!("{}", serde_json::to_string_pretty(&value).unwrap());

    Ok(())
}

fn decode(source: &str, toml_version: TomlVersion) -> Result<Value, anyhow::Error> {
    let p = tombi_parser::parse(source, toml_version);

    if !p.errors.is_empty() {
        for error in p.errors {
            eprintln!("{error}");
        }
        return Err(anyhow::anyhow!(INVALID_MESSAGE));
    }

    let Some(root) = tombi_ast::Root::cast(p.into_syntax_node()) else {
        eprintln!("ast root cast failed");
        return Err(anyhow::anyhow!(INVALID_MESSAGE));
    };

    let root = match root.try_into_document_tree(toml_version) {
        Ok(root) => root,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            return Err(anyhow::anyhow!(INVALID_MESSAGE));
        }
    };

    Ok(root.into_value(toml_version))
}

#[cfg(test)]
macro_rules! test_decode {
    {
        #[test]
        fn $name:ident($source:expr, $toml_version:expr) -> Ok($expected:expr)
    } => {
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>>
        {
            let source = textwrap::dedent($source);
            let value = crate::decode(source.trim(), $toml_version)?;
            pretty_assertions::assert_eq!(
                serde_json::to_string(&value)?,
                serde_json::to_string(&$expected)?
            );

            Ok(())
        }
    };

    {
        #[test]
        fn $name:ident($source:expr) -> Ok($expected:expr)
    } => {
        test_decode! {
            #[test]
            fn $name($source, tombi_toml_version::TomlVersion::default()) -> Ok($expected)
        }
    };
}

#[cfg(test)]
mod test {
    use serde_json::json;

    test_decode! {
        #[test]
        fn check_test_case(
            r#"
            beee = """
            heeee
            geeee\


                    """
            "#
        ) -> Ok(json!(
                        {
           "beee": {"type": "string", "value": "heeee\ngeeee"}
       }
            )
        )
    }
}
