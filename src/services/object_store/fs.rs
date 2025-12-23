use std::{io::ErrorKind, path::PathBuf};

use async_trait::async_trait;
use tokio::fs::{create_dir_all, read, remove_file, try_exists, write};

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
        Ok(read(self.path.join(key)).await?)
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn test_fs_object_store_crud() {
        let path = temp_dir().join(Uuid::new_v4().to_string());
        let contents = "Hello world!".as_bytes();
        let key = "test_key";
        let store = FileSystemObjectStore::new(path).await.unwrap();

        assert_eq!(
            store.get(key).await.unwrap_err().to_string(),
            "No such file or directory (os error 2)"
        );
        store.put(key, contents).await.unwrap();
        let contents = store.get(key).await.unwrap();

        assert_eq!(contents, contents);

        store.delete(key).await.unwrap();

        assert_eq!(
            store.get(key).await.unwrap_err().to_string(),
            "No such file or directory (os error 2)"
        );
    }
}
