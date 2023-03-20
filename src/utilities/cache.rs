use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct Cache<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + PartialOrd,
{
    pub data: HashSet<T>,
    pub path: PathBuf,
    count: usize,
}

impl<T> Cache<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + PartialOrd + Eq + Hash,
{
    pub fn new(cache_file: PathBuf, count: usize) -> Self {
        create_file_if_not_exists(&cache_file).unwrap();
        let cache = File::open(&cache_file).unwrap();
        let cache_reader = std::io::BufReader::new(cache);
        let data = serde_json::from_reader(cache_reader).unwrap_or_default();
        Self {
            data,
            path: cache_file,
            count,
        }
    }

    pub fn add(&mut self, new_data: T) {
        self.data.insert(new_data);
        self.save();
    }

    pub fn add_range(&mut self, new_data: Vec<T>) {
        self.data = self
            .data
            .union(&new_data.into_iter().collect::<HashSet<T>>())
            .cloned()
            .collect();
        self.save();
    }

    pub fn update_range(&mut self, new_data: Vec<T>) {
        self.data.extend(new_data.into_iter());
        self.save();
    }

    pub fn update(&mut self, new_data: T) {
        self.data.insert(new_data);
        self.save();
    }

    pub fn get(&self) -> Vec<T> {
        self.sort()
    }

    fn save(&mut self) {
        let mut cache = File::create(&self.path).unwrap();

        if self.data.len() > self.count {
            let mut data: Vec<T> = self.sort().into_iter().collect();
            data.truncate(self.count);
            self.data = data.into_iter().collect();
        }

        serde_json::to_writer(&mut cache, &self.data).unwrap();
        cache.flush().unwrap();
    }

    fn sort(&self) -> Vec<T> {
        let mut data: Vec<T> = self
            .data
            .iter()
            .cloned()
            .collect::<Vec<T>>()
            .into_iter()
            .collect();

        data.sort_by(|a, b| b.partial_cmp(a).unwrap());
        data
    }
}

fn create_file_if_not_exists(path: &PathBuf) -> Result<(), Error> {
    let file = File::open(path);
    if !file.is_ok() {
        debug!("Creating cache file at {}", path.display());
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir).unwrap();
        File::create(&path)?;
    }

    Ok(())
}
