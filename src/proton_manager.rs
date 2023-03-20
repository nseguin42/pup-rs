use std::collections::HashMap;
use std::path::PathBuf;

use base_url::BaseUrl;
use checksums::Algorithm;
use dirs::cache_dir;

use crate::config::ConfigModule;
use crate::error::Error;
use crate::models::asset::Asset;
use crate::models::release::Release;
use crate::utilities::cache::Cache;
use crate::utilities::downloader::Downloader;
use crate::utilities::downloader::FileGetter;
use crate::utilities::extract;

pub struct ProtonManager {
    pub config: ConfigModule,
    releases_cache: Cache<Release>,
}

impl ProtonManager {
    pub fn new(config: ConfigModule) -> Self {
        let releases_cache_file = cache_dir().unwrap().join("pup-rs").join("releases.json");
        let releases_cache = Cache::<Release>::new(releases_cache_file, 100);

        Self {
            config,
            releases_cache,
        }
    }

    pub async fn get_releases(
        &mut self,
        count: u8,
        installed: bool,
    ) -> Result<Vec<Release>, Error> {
        let releases = match installed {
            true => self.get_installed_releases().await?,
            false => self.fetch_releases(count).await?,
        };

        Ok(releases)
    }

    pub async fn get_installed_releases(&self) -> Result<Vec<Release>, Error> {
        let releases = self
            .releases_cache
            .data
            .iter()
            .filter(|r| r.installed_in.is_some())
            .cloned()
            .collect();

        Ok(releases)
    }

    pub async fn fetch_releases(&mut self, count: u8) -> Result<Vec<Release>, Error> {
        let releases: Vec<Release> = octocrab::instance()
            .repos(self.config.owner.as_str(), self.config.repo.as_str())
            .releases()
            .list()
            .per_page(count)
            .send()
            .await
            .map(|r| r.items)
            .map_err(|e| Error::Api(e.to_string()))?
            .into_iter()
            .map(Release::from)
            .collect();

        self.releases_cache.add_range(releases.clone());
        Ok(releases)
    }

    pub async fn get_release(&self, tag: &str) -> Result<Release, Error> {
        let release = octocrab::instance()
            .repos(self.config.owner.as_str(), self.config.repo.as_str())
            .releases()
            .get_by_tag(tag)
            .await
            .map_err(|e| Error::Api(e.to_string()))?;
        debug!(
            "Found release {} ({}) from {}",
            release.tag_name,
            release.id,
            release.created_at.unwrap().format("%Y-%m-%d")
        );

        Ok(release.into())
    }

    pub async fn install_release(&mut self, tag: &str) -> Result<(), Error> {
        info!("Installing release {}", tag);
        let mut release = self.get_release(tag).await?;
        let downloaded_file = self.download_release(&release).await?;

        info!(
            "Extracting {} to {}",
            downloaded_file.display(),
            self.config.install_dir.display()
        );
        extract::extract(&downloaded_file, &self.config.install_dir)?;

        release.installed_in = Some(self.config.install_dir.clone());
        self.releases_cache.add(release);

        info!("Release {} installed successfully.", tag);
        Ok(())
    }

    async fn download_release(&self, release: &Release) -> Result<PathBuf, Error> {
        let asset = self.get_asset(release).await?;
        let download_url = BaseUrl::try_from(asset.browser_download_url.as_str())?;
        let filename = download_url.path_segments().last().unwrap().to_string();
        debug!("Found asset {} at {}", filename, download_url);

        let (checksum, checksum_algorithm) = self.fetch_checksum(release, &filename).await?;
        let download_path = self.config.cache_dir.join(&filename);
        let cache_dir = self.config.cache_dir.to_str();

        let downloader = Downloader::new(
            Option::from(download_url),
            Option::from(download_path.to_path_buf()),
            cache_dir,
            Option::from(checksum.as_str()),
            Option::from(checksum_algorithm),
            true,
        );
        let file = downloader.get_file().await?;

        Ok(file.into())
    }

    async fn fetch_checksum(
        &self,
        release: &Release,
        filename: &str,
    ) -> Result<(String, Algorithm), Error> {
        let basename = filename.split('.').next().unwrap();
        debug!("Fetching checksum for {}", basename);

        let hash_types = HashMap::from([
            ("sha512sum", Algorithm::SHA2512),
            ("sha256sum", Algorithm::SHA2256),
            ("sha1sum", Algorithm::SHA1),
            ("md5sum", Algorithm::MD5),
        ]);

        let asset_filenames = release
            .assets
            .iter()
            .map(|a| {
                a.browser_download_url
                    .path_segments()
                    .unwrap()
                    .last()
                    .unwrap()
            })
            .collect::<Vec<_>>();
        debug!("Assets: {:?}", asset_filenames);

        let assets_with_same_basename = asset_filenames
            .iter()
            .filter(|a| a.starts_with(basename))
            .collect::<Vec<_>>();

        for (hash_type, algorithm) in hash_types {
            let checksum_asset_idx = assets_with_same_basename
                .iter()
                .position(|a| a.starts_with(basename) && a.ends_with(hash_type));

            if checksum_asset_idx.is_none() {
                continue;
            }

            let checksum_url = release
                .assets
                .get(checksum_asset_idx.unwrap())
                .unwrap()
                .browser_download_url
                .clone();

            let response = reqwest::get(checksum_url).await?;

            let checksum = response
                .text()
                .await?
                .split_whitespace()
                .next()
                .unwrap()
                .to_string();

            return Ok((checksum, algorithm));
        }

        Err(Error::NotFound(
            "Could not find a checksum file in the release".to_string(),
        ))
    }

    async fn get_asset(&self, release: &Release) -> Result<Asset, Error> {
        let asset_types = vec!["tar.gz", "tar.xz"];

        for asset_type in asset_types {
            let asset = release
                .assets
                .iter()
                .find(|a| a.name.ends_with(asset_type))
                .cloned();

            if asset.is_some() {
                return Ok(asset.unwrap());
            }
        }

        Err(Error::NotFound(
            "Could not find a supported asset in the release".to_string(),
        ))
    }
}
