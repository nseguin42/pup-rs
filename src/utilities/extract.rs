use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;
use xz2::read::XzDecoder;

use crate::error::Error;

const SUPPORTED_EXTENSIONS: [&str; 2] = ["gz", "xz"];

pub fn is_supported_extension(extension: &str) -> bool {
    SUPPORTED_EXTENSIONS.contains(&extension)
}

pub fn has_supported_extension(path: &Path) -> bool {
    let extension = path.extension().unwrap();
    is_supported_extension(extension.to_str().unwrap())
}

pub fn extract(archive: &Path, destination: &Path) -> Result<(), Error> {
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

fn unpack_tar_and_save(buffer: Vec<u8>, destination: &Path) -> Result<(), Error> {
    let mut archive = Archive::new(buffer.as_slice());
    archive.unpack(destination)?;
    Ok(())
}

fn save(buffer: Vec<u8>, destination: &Path) -> Result<(), Error> {
    let mut file = File::create(destination)?;
    file.write_all(&buffer)?;
    Ok(())
}

fn extract_gz(archive: &Path, destination: &Path) -> Result<(), Error> {
    let file = File::open(archive)?;
    let mut decoder = GzDecoder::new(file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;

    let url = archive.as_os_str().to_str().unwrap();

    if url.ends_with(".tar.gz") {
        unpack_tar_and_save(buffer, destination)
    } else {
        save(buffer, destination)
    }
}

fn extract_xz(archive: &Path, destination: &Path) -> Result<(), Error> {
    let file = File::open(archive)?;
    let mut decoder = XzDecoder::new(file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;

    let url = archive.as_os_str().to_str().unwrap();

    if url.ends_with(".tar.xz") {
        unpack_tar_and_save(buffer, destination)
    } else {
        save(buffer, destination)
    }
}
