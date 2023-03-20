#[macro_use]
extern crate log;

use clap::Parser;
use prettytable::{row, Row, Table};
use pup_rs::proton_manager::ProtonManager;

use pup_rs::cli;
use pup_rs::cli::{Cli, Command};
use pup_rs::config::Config;
use pup_rs::models::release::Release;

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
    ProtonManager::new(module_name, &module_config)
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

    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row!["Tag", "Date", "Installed in"]);
    for release in releases {
        table.add_row(get_list_table_row(&release));
    }
    table.printstd();
}

fn get_list_table_row(release: &Release) -> Row {
    let date = release.published_at.unwrap().format("%Y-%m-%d");
    row![
        release.tag_name,
        date,
        release
            .installed_in
            .clone()
            .unwrap_or_default()
            .to_str()
            .unwrap()
    ]
}

async fn handle_install(mut pm: ProtonManager, install: cli::Install) {
    pm.install_release(&install.tag).await.unwrap();
}
