use std::path::{Path, PathBuf};

use async_trait::async_trait;
use base_url::BaseUrl;
use checksums::{hash_file, Algorithm};
use reqwest::get;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::error::Error;

pub struct Downloader {
    download: Option<DownloadParams>,
    cache: Option<CacheParams>,
    verify: Option<VerifyParams>,
}

impl Downloader {
    pub fn new(
        download_url: Option<BaseUrl>,
        destination: Option<PathBuf>,
        cache_dir: Option<&str>,
        checksum: Option<&str>,
        checksum_algorithm: Option<Algorithm>,
        remove_failed: bool,
    ) -> Self {
        let filename = destination
            .as_ref()
            .map(|d| d.file_name().unwrap().to_str().unwrap().to_string());

        let download = match (download_url, destination) {
            (Some(url), Some(destination)) => {
                let download = DownloadParams::new(url, destination);
                match download {
                    Ok(d) => Some(d),
                    Err(e) => {
                        error!("Failed to create download params: {}", e);
                        None
                    }
                }
            }
            _ => None,
        };

        let cached_file_path = match (filename, cache_dir) {
            (Some(filename), Some(cache_dir)) => {
                let path = Path::new(cache_dir).join(filename);
                Some(path)
            }
            _ => None,
        };

        let cache = match cached_file_path {
            Some(path) => {
                let cache = CacheParams::new(path);
                Some(cache)
            }
            _ => None,
        };

        let verify = match (checksum, checksum_algorithm) {
            (Some(checksum), Some(checksum_algorithm)) => {
                let verify = VerifyParams::new(checksum, checksum_algorithm, remove_failed);
                Some(verify)
            }
            _ => None,
        };

        Self {
            download,
            cache,
            verify,
        }
    }

    async fn try_get_from_cache(&self) -> Option<&PathBuf> {
        if let Some(cache) = &self.cache {
            debug!(
                "Looking for cached file in: {:?}",
                &self.cache.as_ref().unwrap().cache_path
            );

            let path = cache.get_file().await;
            if path.is_ok() {
                let path = path.unwrap();
                debug!("Found file in cache: {:?}", path);
                return Some(path);
            } else {
                debug!("File not found in cache");
            }
        } else {
            debug!("No cache path provided");
        }
        None
    }

    async fn try_download(&self) -> Result<Option<&PathBuf>, Error> {
        if let Some(download) = &self.download {
            info!("Downloading file...");
            let path = download.get_file().await?;
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
pub trait FileGetter {
    async fn get_file(&self) -> Result<&PathBuf, Error>;
}

#[async_trait]
impl FileGetter for Downloader {
    async fn get_file(&self) -> Result<&PathBuf, Error> {
        let mut path = self.try_get_from_cache().await;
        if path.is_some() {
            info!("Found file in cache, skipping download.");
        } else {
            path = self.try_download().await?;
        }

        if path.is_none() {
            return Err(Error::NotFound(String::from(
                path.unwrap().to_str().unwrap(),
            )));
        }

        let path = path.unwrap();
        if let Some(verify) = &self.verify {
            verify.verify(path).await?;
        } else {
            warn!("No checksum provided, skipping verification.");
        }

        Ok(path)
    }
}

struct DownloadParams {
    url: BaseUrl,
    destination: PathBuf,
}

impl DownloadParams {
    fn new(url: BaseUrl, destination: PathBuf) -> Result<Self, Error> {
        Ok(Self { url, destination })
    }
}

#[async_trait]
impl FileGetter for DownloadParams {
    async fn get_file(&self) -> Result<&PathBuf, Error> {
        let mut file = File::create(self.destination.clone()).await?;
        let mut response = get(self.url.as_str()).await?;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        Ok(&self.destination)
    }
}

struct CacheParams {
    cache_path: PathBuf,
}

impl CacheParams {
    fn new(cache_path: PathBuf) -> Self {
        Self { cache_path }
    }

    fn file_exists(&self) -> bool {
        debug!("Checking if file exists: {:?}", self.cache_path);
        self.cache_path.exists()
    }
}

#[async_trait]
impl FileGetter for CacheParams {
    async fn get_file(&self) -> Result<&PathBuf, Error> {
        if self.file_exists() {
            Ok(&self.cache_path)
        } else {
            Err(Error::CacheFileNotFound(
                self.cache_path.clone().to_str().unwrap().to_string(),
            ))
        }
    }
}

struct VerifyParams {
    checksum: String,
    checksum_algorithm: Algorithm,
    remove_failed: bool,
}

impl VerifyParams {
    fn new(checksum: &str, checksum_algorithm: Algorithm, remove_failed: bool) -> Self {
        let checksum = String::from(checksum);
        Self {
            checksum,
            checksum_algorithm,
            remove_failed,
        }
    }

    pub async fn verify(&self, file_path: &Path) -> Result<(), Error> {
        info!("Verifying file...");
        debug!("Checksum: {}", self.checksum);

        let checksum = hash_file(file_path, self.checksum_algorithm);
        if !checksum.eq_ignore_ascii_case(&self.checksum) {
            debug!("Failed to verify file: {}", file_path.display());
            if self.remove_failed {
                debug!("Removing file: {}", file_path.display());
                tokio::fs::remove_file(file_path).await.unwrap();
            }
            return Err(Error::Mismatch {
                expected: self.checksum.clone(),
                actual: checksum,
            });
        }

        info!("Checksums match.");
        Ok(())
    }
}
