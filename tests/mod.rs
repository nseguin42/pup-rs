use pup_rs::config::{Config};
use pup_rs::proton_manager::ProtonManager;
use tokio::test;

fn get_manager() -> ProtonManager {
    let config = Config::new(Option::from("tests/config.test.toml".to_string()))
        .modules
        .iter()
        .next()
        .unwrap()
        .1
        .clone();
    ProtonManager::new(config)
}

#[test]
async fn test_get_releases() {
    let mut manager = get_manager();
    let releases = manager.get_releases(1, false).await.unwrap();
    assert!(!releases.is_empty());
}
