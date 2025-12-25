use std::{
    env::temp_dir,
    fs::{create_dir, remove_dir_all},
    path::{Path, PathBuf},
};

use chromiumoxide::{BrowserFetcher, BrowserFetcherOptions};
use tokio::fs::create_dir_all;
use uuid::Uuid;

use crate::types::error::AppError;

pub async fn download_browser(path: Option<&PathBuf>) -> Result<PathBuf, AppError> {
    let path = match path {
        Some(p) => p.clone(),
        None => {
            let mut p = get_temp_root();
            p.push("browser");

            p
        }
    };

    create_dir_all(&path).await?;

    let fetcher = BrowserFetcher::new(BrowserFetcherOptions::builder().with_path(path).build()?);
    let info = fetcher.fetch().await?;

    Ok(info.executable_path)
}

pub fn get_temp_root() -> PathBuf {
    let mut p = temp_dir();
    p.push("aetherscope");

    p
}

fn unique_temp_dir() -> Result<PathBuf, AppError> {
    let dir = get_temp_root().join(Uuid::new_v4().to_string());
    create_dir(&dir)?;

    Ok(dir)
}

pub struct TempDir(PathBuf);
impl TempDir {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self(unique_temp_dir()?))
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.0);
    }
}
