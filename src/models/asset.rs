use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: Url,
    pub updated_at: DateTime<Utc>,
}

impl From<octocrab::models::repos::Asset> for Asset {
    fn from(asset: octocrab::models::repos::Asset) -> Self {
        Self {
            name: asset.name,
            browser_download_url: asset.browser_download_url,
            updated_at: asset.updated_at,
        }
    }
}

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.updated_at.partial_cmp(&other.updated_at)
    }
}
