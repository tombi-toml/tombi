use clap::CommandFactory;

/// Generate shell completion.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Shell to generate completion for
    #[arg(value_enum)]
    shell: clap_complete::Shell,
}

pub fn run(args: Args) -> Result<(), crate::Error> {
    let mut cmd = crate::app::Args::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(args.shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}
