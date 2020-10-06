use anyhow::anyhow;
use structopt::clap::Shell;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "SSH Key Sync",
    about = "A command line client and service for keeping SHH keys up to date with a list Ex: Github."
)]
enum Cli {
    /// The username to use
    #[structopt(name = "get")]
    Get {
        /// The username to pull from
        #[structopt(name = "username")]
        username: String,

        #[structopt(short, long)]
        github: bool,
    },
}

fn main() -> anyhow::Result<()> {
    Cli::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");

    let cli = Cli::from_args();
    return Ok(());
}
