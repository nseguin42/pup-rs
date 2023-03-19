use crate::config::Config;
use crate::error::Error;
use crate::utilities::extract;
use checksums::{hash_file, Algorithm};
use octocrab::models::repos::Release;
use reqwest::Url;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct ProtonManager {
    pub config: Config,
}

impl ProtonManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn fetch_releases(&self, count: u8) -> Result<Vec<Release>, Error> {
        octocrab::instance()
            .repos(self.config.owner.as_str(), self.config.repo.as_str())
            .releases()
            .list()
            .per_page(count)
            .send()
            .await
            .map(|r| r.items)
            .map_err(|e| Error::Api(e.to_string()))
    }

    pub async fn get_release(&self, tag: &str) -> Result<Release, Error> {
        octocrab::instance()
            .repos(self.config.owner.as_str(), self.config.repo.as_str())
            .releases()
            .get_by_tag(tag)
            .await
            .map_err(|e| Error::Api(e.to_string()))
    }

    pub async fn install_proton(
        &self,
        tag: &str,
        use_cache: bool,
        verify: bool,
    ) -> Result<(), Error> {
        let download_path = self.download_proton(tag, use_cache, verify).await?;
        let extract_path = self.config.install_dir.join(tag);
        extract::extract(&download_path, &extract_path)?;
        Ok(())
    }

    async fn download_proton(
        &self,
        tag: &str,
        use_cache: bool,
        verify: bool,
    ) -> Result<PathBuf, Error> {
        let release = self.get_release(tag).await?;

        let asset = release
            .assets
            .iter()
            .find(|a| a.name.contains("tar.gz"))
            .ok_or(Error::NotFound(
                "Could not find a tar.gz asset in the release".to_string(),
            ))?;

        let tag = release.tag_name.clone();
        let download_path_str = self
            .config
            .cache_dir
            .join(format!("{}.tar.gz", tag))
            .to_str()
            .unwrap()
            .to_string();
        let download_path = Path::new(download_path_str.as_str());

        let use_cached_file = use_cache && download_path.exists();
        if !use_cached_file {
            let url = asset.browser_download_url.clone();
            info!("Downloading release {} from {}", tag, url);
            self.download_and_save(url, download_path).await?;
            debug!("Finished downloading {}", tag);
        } else {
            info!("Using cached download for {}", tag);
        }

        if verify {
            self.verify_download(release, download_path, !use_cached_file)
                .await?;
        } else {
            warn!("Skipping verification of download");
        }

        Ok(download_path.to_path_buf())
    }

    async fn verify_download(
        &self,
        release: Release,
        download_path: &Path,
        remove_on_error: bool,
    ) -> Result<(), Error> {
        debug!("Attempting to verify download");
        let downloaded_hash_file = self.fetch_hash_file(&release).await?;
        debug!("Hash file: {}", downloaded_hash_file);

        let mut hash_file_split = downloaded_hash_file.split_whitespace().rev();
        let mut err = None;
        let expected_filename = hash_file_split.next().ok_or(Error::NotFound(
            "Could not find filename in hash file".to_string(),
        ))?;
        debug!("Expected filename: {}", expected_filename);
        let actual_filename = download_path
            .file_name()
            .ok_or(Error::NotFound(
                "Could not find filename in download path".to_string(),
            ))?
            .to_str()
            .unwrap();
        debug!("Actual filename: {}", actual_filename);

        if !expected_filename.eq_ignore_ascii_case(actual_filename) {
            error!(
                "Filename mismatch: {} != {}",
                expected_filename, actual_filename
            );
            err = Err(Error::Mismatch {
                expected: expected_filename.to_string(),
                actual: actual_filename.to_string(),
            })?;
        }

        let expected_hash = hash_file_split.next().ok_or(Error::NotFound(
            "Could not find hash in hash file".to_string(),
        ))?;
        debug!("Expected hash: {}", expected_hash);

        let actual_hash = hash_file(download_path, Algorithm::SHA2512);
        debug!("Actual hash: {}", actual_hash);

        let hashes_match = actual_hash.eq_ignore_ascii_case(&expected_hash);

        if !hashes_match {
            error!("Hash mismatch");
            err = Err(Error::Mismatch {
                expected: expected_hash.to_string(),
                actual: actual_hash,
            })?;
        }

        if err.is_some() {
            if remove_on_error {
                std::fs::remove_file(download_path)?;
            }
            err.unwrap()
        } else {
            Ok(())
        }
    }

    async fn fetch_hash_file(&self, release: &Release) -> Result<String, Error> {
        debug!("Fetching hash file for release");
        let asset = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".sha512sum"))
            .ok_or(Error::NotFound(
                "Could not find a .sha512 asset in the release".to_string(),
            ))?;

        let url = asset.browser_download_url.clone();
        let response = reqwest::get(url).await?;
        let hash_file = response.text().await?;
        Ok(hash_file)
    }

    async fn download_and_save(&self, url: Url, output_path: &Path) -> Result<(), Error> {
        match reqwest::get(url).await {
            Ok(response) => {
                let mut file = std::fs::File::create(output_path)?;
                file.write_all(&response.bytes().await?)?;
                Ok(())
            }
            Err(e) => Err(Error::Api(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, GzBuilder};
    use std::assert_eq;
    use std::fs::File;

    const TEST_REPO: &str = "proton-ge-custom";
    const TEST_OWNER: &str = "GloriousEggroll";
    const TEST_TAG: &str = "GE-Proton7-51";
    const TEST_DOWNLOAD_DIR: &str = "/tmp/protonup/";
    const TEST_EXTRACT_PATH: &str = "/tmp/protonup/extracted";

    fn get_test_config() -> Config {
        Config {
            install_dir: Default::default(),
            cache_dir: Default::default(),
            repo: "".to_string(),
            owner: "".to_string(),
        }
    }

    fn get_test_manager() -> ProtonManager {
        ProtonManager {
            config: get_test_config(),
        }
    }

    fn create_test_download() -> Result<(), Error> {
        let filename = format!("{}.tar.gz", TEST_TAG);
        let download_path = Path::new(TEST_DOWNLOAD_DIR).join(filename.clone());

        let f = File::create(download_path.clone())?;
        let mut gz = GzBuilder::new()
            .filename("hello_world.txt")
            .comment("test file, please delete")
            .write(f, Compression::default());
        gz.write_all(b"hello world")?;
        gz.finish().expect("Failed to finish writing tar.gz file");

        let hash = hash_file(&download_path, Algorithm::SHA2512);

        let mut h = File::create(download_path.with_extension("sha512sum"))?;
        h.write_all(format!("{}  {}", hash, filename).as_bytes())?;

        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_releases() {
        let manager = get_test_manager();
        let releases = manager.fetch_releases(10).await;
        assert!(releases.is_ok());
        assert_eq!(releases.unwrap().len(), 10);
    }

    #[tokio::test]
    async fn test_get_release() {
        let manager = get_test_manager();
        let release = manager.get_release(TEST_TAG).await;
        assert!(release.is_ok());
        assert_eq!(release.unwrap().tag_name, TEST_TAG);
    }

    #[tokio::test]
    async fn test_download_proton() {
        let manager = get_test_manager();
        let release = manager.get_release(TEST_TAG).await.unwrap();
        manager
            .download_proton(TEST_TAG, false, true)
            .await
            .unwrap();

        assert!(Path::new(TEST_DOWNLOAD_DIR)
            .join(format!("{}.tar.gz", TEST_TAG))
            .exists());
        assert!(Path::new(TEST_DOWNLOAD_DIR)
            .join(format!("{}.tar.gz.sha512sum", TEST_TAG))
            .exists());
    }
}
