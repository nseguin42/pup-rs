use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version)]
#[command(
    about = "pup-rs: a CLI tool to manage installed Proton versions.",
    author = "nseguin42"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[clap(long = "config")]
    #[clap(help = "The path to the config file.")]
    pub config_path: Option<String>,

    #[clap(long = "module")]
    #[clap(help = "The name of the config module to use. Defaults to the first module defined.")]
    pub module: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(about = "List releases.")]
    List(List),

    #[clap(about = "Install a release.")]
    Install(Install),
}

#[derive(Debug, Args)]
pub struct List {
    #[arg(short = 'i', long, default_value = "false")]
    #[clap(help = "List only the releases installed in the install directory.")]
    pub installed: bool,

    #[arg(short, long)]
    #[clap(help = "The number of releases to list.")]
    pub count: Option<u8>,
}

#[derive(Debug, Args)]
pub struct Install {
    pub tag: String,

    #[arg(long = "cache", default_value = "true")]
    #[clap(help = "Check if the release has already been downloaded to the cache.")]
    pub use_cache: bool,

    #[arg(long = "verify", default_value = "true")]
    #[clap(help = "Verify the download with the checksum provided in the release.")]
    pub verify_download: bool,

    #[clap(long = "install-dir")]
    #[clap(
        help = "The directory to install the release to. Defaults to the value in the config file."
    )]
    pub install_dir: Option<String>,

    #[clap(long = "cache-dir")]
    #[clap(
        help = "The directory to cache the release in. Defaults to the value in the config file."
    )]
    pub cache_dir: Option<String>,
}

#[derive(Debug, Args)]
pub struct Remove {
    tag: String,
}
