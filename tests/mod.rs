use pup_rs::config::Config;
use pup_rs::proton_manager::ProtonManager;
use tokio::test;

fn get_manager() -> ProtonManager {
    let config = Config::new(Option::from("tests/config.test.toml".to_string()));

    let module = config.modules.iter().next().unwrap().clone();
    ProtonManager::new(module.0.clone(), module.1)
}

#[test]
async fn test_get_releases() {
    let mut manager = get_manager();
    let releases = manager.get_releases(1, false).await.unwrap();
    assert!(!releases.is_empty());
}
