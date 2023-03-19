use crate::error::Error;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tar::Archive;

pub fn extract(archive: &Path, destination: &Path) -> Result<(), Error> {
    let extension = archive.extension().unwrap();
    match extension.to_str().unwrap() {
        "gz" => extract_gz(archive, destination),
        _ => Err(Error::FileTypeNotSupported(
            extension.to_str().unwrap().to_string(),
        )),
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

    return if url.ends_with(".tar.gz") {
        unpack_tar_and_save(buffer, destination)
    } else {
        save(buffer, destination)
    };
}
