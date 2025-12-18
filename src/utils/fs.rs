use std::{
    env::temp_dir,
    path::{Path, PathBuf},
};

use chromiumoxide::{BrowserFetcher, BrowserFetcherOptions};
use tokio::fs::create_dir_all;

use crate::types::error::AppError;

pub async fn download_browser(path: Option<&PathBuf>) -> Result<PathBuf, AppError> {
    let path = match path {
        Some(p) => p,
        None => {
            let mut p = get_temp_root();
            p.push("browser");

            p
        }
    };

    create_dir_all(path).await?;

    let fetcher = BrowserFetcher::new(BrowserFetcherOptions::builder().with_path(path).build()?);
    let info = fetcher.fetch().await?;

    Ok(info.executable_path)
}

pub fn get_temp_root() -> PathBuf {
    let mut p = temp_dir();
    p.push("aetherscope");

    p
}
