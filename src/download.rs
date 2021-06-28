use anyhow::Result;
use std::path::Path;
use reqwest::Client;
use sha1::{Sha1, Digest};
use std::fs;
use std::io::Write;
use crate::util::{complete, info};

#[derive(thiserror::Error, Debug)]
pub enum DownloadError {
    #[error("expected file hash: {0} but downloaded file with hash: {1}")]
    InvalidHash(String, String),
}

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn download_if_hash_invalid(&self, output: &Path, url: &str, hash: &str) -> Result<()> {
        if output.exists() {
            let data = fs::read(output)?;
            let file_hash = Sha1::digest(&*data);
            let file_hash = format!("{:02x}", file_hash);
            if file_hash == hash {
                complete(&*format!("{:?} is already ok!", output));
                return Ok(());
            }
        }

        info(&*format!("Downloading {}...", url));
        let data = self.client.get(url).send().await?.bytes().await?;
        let digest = Sha1::digest(&*data);
        let download_hash = format!("{:02x}", digest);
        if download_hash != hash {
            return Err(DownloadError::InvalidHash(hash.to_string(), download_hash).into());
        }

        let parent = output.parent().expect("File does not have a parent");
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        let mut output_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output)?;
        output_file.write_all(&*data)?;

        Ok(())
    }
}
