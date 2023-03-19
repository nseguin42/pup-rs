mod cli;
mod config;
mod constants;
mod error;
mod proton_manager;
mod utilities;

#[macro_use]
extern crate log;

use crate::cli::{Cli, Command};
use crate::config::Config;
use crate::proton_manager::ProtonManager;
use clap::Parser;

#[tokio::main]
async fn main() {
    setup_logger();
    let cli = Cli::parse();
    debug!("CLI: {:?}", cli);

    let config = Config::new(cli.config_path.clone());
    let pm = get_proton_manager(config);
    handle_command(cli, pm).await;
}

fn setup_logger() {
    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Debug)
        .init();
}

fn get_proton_manager(config: Config) -> ProtonManager {
    ProtonManager::new(config, "proton-ge-custom", "GloriousEggroll")
}

pub async fn handle_command(cli: Cli, pm: ProtonManager) {
    match cli.command {
        Some(command) => match command {
            Command::List(list) => handle_list(pm, list).await,
            Command::Install(install) => handle_install(pm, install).await,
        },
        None => println!("No command provided"),
    }
}

async fn handle_list(pm: ProtonManager, list: cli::List) {
    let releases = pm.fetch_releases(list.count.unwrap_or(10)).await.unwrap();
    for release in releases {
        info!("{}", release.tag_name);
    }
}

async fn handle_install(pm: ProtonManager, install: cli::Install) {
    pm.install_proton(&install.tag, install.use_cache, install.verify_download)
        .await
        .unwrap();
}

async fn handle_remove(pm: ProtonManager, remove: crate::cli::Remove) {
    //let release = pm.remove_release(&remove.tag).await.unwrap();
}
