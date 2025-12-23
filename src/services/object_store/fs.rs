use std::{io::ErrorKind, path::PathBuf};

use async_trait::async_trait;
use tokio::fs::{create_dir_all, remove_file, try_exists, write};

use crate::types::{error::AppError, traits::object_store::ObjectStore};

pub struct FileSystemObjectStore {
    path: PathBuf,
}

impl FileSystemObjectStore {
    pub async fn new(path: PathBuf) -> Result<Self, AppError> {
        create_dir_all(path.clone()).await?;

        Ok(Self { path })
    }
}

#[async_trait]
impl ObjectStore for FileSystemObjectStore {
    async fn get(&self, key: &str) -> Result<Vec<u8>, AppError> {
        unimplemented!();
    }
    async fn put(&self, key: &str, data: &[u8]) -> Result<(), AppError> {
        Ok(write(self.path.join(key), data).await?)
    }
    async fn delete(&self, key: &str) -> Result<(), AppError> {
        match remove_file(self.path.join(key)).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
    async fn exists(&self, key: &str) -> Result<bool, AppError> {
        Ok(try_exists(self.path.join(key)).await?)
    }
}
