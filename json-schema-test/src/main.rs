use anyhow::Result;
use clap::Parser;
use json_schema_test::{
    Draft, RunOptions, ensure_suite, metaschema_cache_dir, print_summary, run_suite, suite_commit,
};

#[derive(Debug, Parser)]
#[command(
    name = "json-schema-test",
    about = "Run JSON-Schema-Test-Suite against Tombi (Definition A: required tests, TOML-representable instances)"
)]
struct Args {
    /// Drafts to run (default: all of draft7, draft2019-09, draft2020-12).
    #[arg(long = "draft", value_enum)]
    drafts: Vec<Draft>,

    /// Only run suite files whose path contains this substring.
    #[arg(long)]
    filter: Option<String>,

    /// Stop after this many failures.
    #[arg(long)]
    max_failures: Option<usize>,

    /// How many failure details to print.
    #[arg(long, default_value_t = 50)]
    show_failures: usize,

    /// Exit 0 even when there are failures (useful while closing gaps).
    #[arg(long, default_value_t = false)]
    allow_fail: bool,

    /// Fetch/update the vendored suite and exit.
    #[arg(long, default_value_t = false)]
    fetch_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let suite_root = ensure_suite()?;
    println!("Using JSON-Schema-Test-Suite @ {}", suite_commit());

    // Point SchemaStore's offline HTTP cache at the vendored official metaschemas.
    // SAFETY: this binary is a dedicated suite runner; the env var is process-scoped.
    unsafe {
        std::env::set_var("TOMBI_CACHE_HOME", metaschema_cache_dir());
    }

    if args.fetch_only {
        println!("Suite fetched at {}", suite_root.display());
        println!(
            "Metaschema cache ready at {}",
            metaschema_cache_dir().display()
        );
        return Ok(());
    }

    let drafts = if args.drafts.is_empty() {
        Draft::all().to_vec()
    } else {
        args.drafts
    };

    let options = RunOptions {
        drafts,
        filter: args.filter,
        max_failures: args.max_failures,
    };

    let summary = run_suite(&suite_root, &options).await?;
    print_summary(&summary, args.show_failures);

    if summary.failed > 0 && !args.allow_fail {
        std::process::exit(1);
    }
    Ok(())
}
