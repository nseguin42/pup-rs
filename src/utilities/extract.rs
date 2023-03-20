use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use tar::Archive;
use tokio::io::AsyncBufReadExt;
use xz2::read::XzDecoder;

use crate::error::Error;

const SUPPORTED_EXTENSIONS: [&str; 2] = ["gz", "xz"];

pub fn is_supported_extension(extension: &str) -> bool {
    SUPPORTED_EXTENSIONS.contains(&extension)
}

/// Extracts an archived/compressed file, returning a list of files extracted.
pub fn extract(archive: &Path, destination: &Path) -> Result<Vec<String>, Error> {
    let extension = archive.extension().unwrap().to_str().unwrap();

    if !(is_supported_extension(extension)) {
        return Err(Error::FileTypeNotSupported(
            archive.extension().unwrap().to_str().unwrap().to_string(),
        ));
    }

    if !destination.exists() {
        std::fs::create_dir_all(destination)?;
    }

    match extension {
        "gz" => extract_gz(archive, destination),
        "xz" => extract_xz(archive, destination),
        _ => Err(Error::FileTypeNotSupported(extension.to_string())),
    }
}

fn unpack_tar_and_save(buffer: Vec<u8>, destination: &Path) -> Result<Vec<String>, Error> {
    let mut archive = Archive::new(buffer.as_slice());
    let mut contents = Vec::new();

    archive
        .entries()?
        .filter_map(|e| e.ok())
        .map(|mut entry| -> Result<PathBuf, Error> {
            let path = entry
                .path()?
                .components()
                .fold(destination.to_path_buf(), |mut path, c| {
                    path.push(c.as_os_str());
                    path
                });
            entry.unpack(&path)?;
            Ok(path)
        })
        .filter_map(|e| e.ok())
        .for_each(|path| {
            contents.push(
                path.strip_prefix(destination)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
        });

    contents.retain(|c| c.split('/').count() == 1);
    Ok(contents)
}

fn save(buffer: Vec<u8>, destination: &Path) -> Result<String, Error> {
    let mut file = File::create(destination)?;
    file.write_all(&buffer)?;
    Ok(destination
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string())
}

fn extract_gz(archive: &Path, destination: &Path) -> Result<Vec<String>, Error> {
    let file = File::open(archive)?;
    let mut decoder = GzDecoder::new(file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;

    let url = archive.as_os_str().to_str().unwrap();

    if url.ends_with(".tar.gz") {
        unpack_tar_and_save(buffer, destination)
    } else {
        Ok(vec![save(buffer, destination).unwrap()])
    }
}

fn extract_xz(archive: &Path, destination: &Path) -> Result<Vec<String>, Error> {
    let file = File::open(archive)?;
    let mut decoder = XzDecoder::new(file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;

    let url = archive.as_os_str().to_str().unwrap();

    if url.ends_with(".tar.xz") {
        unpack_tar_and_save(buffer, destination)
    } else {
        Ok(vec![save(buffer, destination).unwrap()])
    }
}
