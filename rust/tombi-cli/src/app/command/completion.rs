use std::io;

use clap::CommandFactory;

use crate::args::{Args, CompletionArgs};

pub fn run(args: CompletionArgs) -> Result<(), crate::Error> {
    let mut cmd = Args::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(args.shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}
