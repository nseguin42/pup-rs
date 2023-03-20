use crate::models::asset::Asset;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Release {
    pub name: Option<String>,
    pub tag_name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub assets: Vec<Asset>,
    pub installed_in: Option<PathBuf>,
}

impl From<octocrab::models::repos::Release> for Release {
    fn from(release: octocrab::models::repos::Release) -> Self {
        Self {
            name: release.name,
            tag_name: release.tag_name,
            published_at: release.published_at,
            created_at: release.created_at,
            assets: release.assets.into_iter().map(Asset::from).collect(),
            installed_in: None,
        }
    }
}

impl PartialEq for Release {
    fn eq(&self, other: &Self) -> bool {
        self.tag_name == other.tag_name && self.published_at == other.published_at
    }
}

impl PartialOrd for Release {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.published_at.partial_cmp(&other.published_at)
    }
}

impl Eq for Release {}

impl Hash for Release {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag_name.hash(state);
        self.published_at.hash(state);
    }
}
