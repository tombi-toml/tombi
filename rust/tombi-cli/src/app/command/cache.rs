/// Manage the cache
#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    command: CacheCommand,
}

#[derive(clap::Subcommand)]
enum CacheCommand {
    /// Remove all cached files
    Clean,
    /// Print the cache directory path
    Dir,
}

pub fn run(args: Args) -> Result<(), crate::Error> {
    match args.command {
        CacheCommand::Clean => {
            super::runtime(true)?.block_on(tombi_cache::clean_cache())?;
        }
        CacheCommand::Dir => println!("{}", tombi_cache::try_get_tombi_cache_dir_path()?.display()),
    }
    Ok(())
}
