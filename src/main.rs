#[macro_use]
extern crate log;

use clap::Parser;
use pup_rs::proton_manager::ProtonManager;

use pup_rs::cli;
use pup_rs::cli::{Cli, Command};
use pup_rs::config::Config;

#[tokio::main]
async fn main() {
    setup_logger();
    let cli = Cli::parse();
    debug!("CLI: {:?}", cli);

    let config = Config::new(cli.config_path.clone());
    handle_command(cli, config).await;
}

fn setup_logger() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();
}

fn get_proton_manager(config: Config, maybe_module_name: Option<String>) -> ProtonManager {
    let (module_name, module_config) = match maybe_module_name {
        Some(module_name) => {
            let module_config = match config.modules.get(&module_name) {
                Some(config) => config.clone(),
                None => {
                    error!("Module {} not found in config.", module_name);
                    std::process::exit(1);
                }
            };
            (module_name, module_config)
        }
        None => {
            let (module_name, module_config) = config.modules.iter().next().unwrap();
            (module_name.clone(), module_config.clone())
        }
    };

    info!(
        "Using config module \"{}\" defined in {}.",
        module_name,
        config.path.to_str().unwrap()
    );
    ProtonManager::new(module_config)
}

pub async fn handle_command(cli: Cli, config: Config) {
    let pm = get_proton_manager(config, cli.module);

    match cli.command {
        Some(command) => match command {
            Command::List(list) => handle_list(pm, list).await,
            Command::Install(install) => handle_install(pm, install).await,
        },
        None => check_for_updates(pm).await,
    }
}

async fn check_for_updates(mut pm: ProtonManager) {
    let releases = pm.get_releases(1, false).await.unwrap();
    if releases.is_empty() {
        error!("No releases found.");
        return;
    }

    let latest_release = releases.first().unwrap();
    let installed_releases = pm.get_releases(1, true).await.unwrap();
    if installed_releases.is_empty() {
        info!(
            "The latest release {} from {} is not installed.",
            latest_release.tag_name,
            latest_release.created_at.unwrap().format("%Y-%m-%d")
        );
        return;
    }

    let installed_release = installed_releases.first().unwrap();
    if latest_release.tag_name != installed_release.tag_name {
        info!(
            "A new release is available: {} (installed: {})",
            latest_release.tag_name, installed_release.tag_name
        );
    } else {
        info!(
            "You are running the latest release: {}",
            latest_release.tag_name
        );
    }
}

async fn handle_list(mut pm: ProtonManager, list: cli::List) {
    let releases = pm
        .get_releases(list.count.unwrap_or(10), list.installed)
        .await
        .unwrap();

    if releases.is_empty() {
        info!("No releases found.");
        return;
    }

    info!(
        "The following releases are {} @{}/{}:",
        if list.installed {
            "installed from"
        } else {
            "available at"
        },
        pm.config.owner,
        pm.config.repo
    );

    // Right-align dates.
    let max_tag_length = releases.iter().map(|r| r.tag_name.len()).max().unwrap_or(0);
    info!("{}{}{}", "Tag", " ".repeat(max_tag_length), "Date");
    info!(
        "{}",
        "-".repeat("Tag".len() + max_tag_length + "Date".len() + 1)
    );
    for release in releases {
        let spaces = " ".repeat(max_tag_length - release.tag_name.len() + 1);
        info!(
            "{}{}{}",
            release.tag_name,
            spaces,
            release.published_at.unwrap().format("%Y-%m-%d")
        );
    }
}

async fn handle_install(mut pm: ProtonManager, install: cli::Install) {
    pm.install_release(&install.tag).await.unwrap();
}
