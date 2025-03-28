//! Base implementation of a File Database for Struct

use crate::powens::{HasId, Sortable};
use serde_json;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, MutexGuard};
use tracing::{debug, info};

#[derive(Clone)]
pub struct StructFileDb<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
{
    db: Arc<Mutex<BaseStructFileDb<T>>>,
}

impl<T> StructFileDb<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
{
    pub fn new(file_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(StructFileDb::<T> {
            db: Arc::new(Mutex::new(BaseStructFileDb::<T>::new(file_path)?)),
        })
    }

    pub fn save(&self, data: Vec<T>) -> Result<(), Box<dyn std::error::Error>> {
        let mut mutex = self.db.lock().unwrap();
        mutex.data = data;
        mutex.save()
    }

    pub fn reload(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut mutex = self.db.lock().unwrap();
        mutex.reload()
    }

    pub fn data(&self) -> Vec<T> {
        let mutex = self.db.lock().unwrap();
        mutex.data.clone()
    }

    pub fn is_data_empty(&self) -> bool {
        let mutex = self.db.lock().unwrap();
        mutex.data.is_empty()
    }
}

impl<T> StructFileDb<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + HasId + Sortable,
{
    fn sort_and_save(
        &self,
        mutex: &mut MutexGuard<BaseStructFileDb<T>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        mutex
            .data
            .sort_by(|a, b| a.sortable_value().cmp(&b.sortable_value()));
        mutex.save()
    }

    pub fn find_by_id(&self, id: u64) -> Option<T> {
        let mutex = self.db.lock().unwrap();
        mutex.data.iter().find(|x| x.id() == id).cloned()
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut mutex = self.db.lock().unwrap();
        mutex.data.retain(|x| x.id() != id);
        self.sort_and_save(&mut mutex)
    }

    pub fn upsert(&self, data: T) -> Result<(), Box<dyn std::error::Error>> {
        let mut mutex = self.db.lock().unwrap();
        let index = mutex.data.iter().position(|x| x.id() == data.id());
        if let Some(index) = index {
            debug!(
                "Update {} with id {}", 
                std::any::type_name::<T>(), 
                &data.id()
            );
            mutex.data[index] = data;
        } else {
            debug!(
                "Insert {} with id {}", 
                std::any::type_name::<T>(), 
                &data.id());
            mutex.data.push(data);
        }
        self.sort_and_save(&mut mutex)
    }
}

struct BaseStructFileDb<T: serde::Serialize + for<'de> serde::Deserialize<'de>> {
    file_path: String,
    data: Vec<T>,
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> BaseStructFileDb<T> {
    fn new(file_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut content = String::new();

        if !fs::exists(&file_path)? {
            // split and get folder, create folder if necessary
            let folder_path = file_path.split("/").collect::<Vec<&str>>()
                [..(file_path.split("/").count() - 1)]
                .join("/");
            if !folder_path.is_empty() && !fs::exists(&folder_path)? {
                fs::create_dir_all(&folder_path)?;
                info!("Created folder: {}", folder_path);
            }

            File::create(&file_path)?;
            info!("Created file: {}", file_path);
        } else {
            let mut file = File::open(&file_path)?;
            file.read_to_string(&mut content)?;
        } // file closed

        let data: Vec<T> = if content.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&content)?
        };

        Ok(BaseStructFileDb::<T> { file_path, data })
    }

    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.data)?;

        let tmp_path = format!("{}.tmp", &self.file_path);
        let mut file = File::create(&tmp_path)?; // this truncates the exiting file if any
        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        fs::rename(&tmp_path, &self.file_path)?; // this replaces the existing file

        info!("Saved file: {}", self.file_path);

        Ok(())
    }

    fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !fs::exists(&self.file_path)? {
            return if self.data.is_empty() {
                Ok(())
            } else {
                Err(Box::from("File does not exist and data is not empty"))
            };
        }

        let content = fs::read_to_string(&self.file_path)?;
        self.data = if content.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&content)?
        };

        Ok(())
    }
}
