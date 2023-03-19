use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version)]
#[command(
    about = "proton-updater: a CLI tool to manage installed Proton versions.",
    author = "nseguin42"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[clap(long = "config")]
    pub config_path: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    List(List),
    Install(Install),
}

#[derive(Debug, Args)]
pub struct List {
    #[arg(short = 'i', long, default_value = "false")]
    pub installed: bool,

    #[arg(short, long)]
    pub count: Option<u8>,
}

#[derive(Debug, Args)]
pub struct Install {
    pub tag: String,

    #[arg(long = "cache", default_value = "true")]
    pub use_cache: bool,

    #[arg(long = "verify", default_value = "true")]
    pub verify_download: bool,

    #[clap(long = "install-dir")]
    pub install_dir: Option<String>,

    #[clap(long = "cache-dir")]
    pub cache_dir: Option<String>,
}

#[derive(Debug, Args)]
pub struct Remove {
    tag: String,
}
