use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use reqwest::Client;
use anyhow::Result;
use crate::util::USER_AGENT;
use crate::util::error;

const MODRINTH_API: &str = "https://api.modrinth.com/api/v1";
const MODRINTH_STAGING_API: &str = "https://staging-api.modrinth.com/api/v1";

#[derive(Deserialize, Debug)]
pub struct ModrinthMod {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub versions: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModrinthVersion {
    pub id: String,
    pub mod_id: String,
    pub name: String,
    pub files: Vec<ModrinthVersionFile>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub date_published: DateTime<Utc>,
}

impl ModrinthVersion {
    fn resolve_file(&self) -> Option<ModrinthVersionFile> {
        self.files.iter().find(|v| v.primary).cloned()
            .or_else(|| self.files.first().cloned())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModrinthVersionFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub hashes: HashMap<String, String>,
}

pub struct ModrinthClient {
    staging: bool,
    client: Client,
}

impl ModrinthClient {
    pub fn new(staging: bool) -> Result<Self> {
        Ok(Self {
            staging,
            client: Client::builder()
                .connection_verbose(true)
                .user_agent(USER_AGENT)
                .build()?
        })
    }

    pub async fn resolve_mod(&self, identifier: &str, predicate: &dyn Fn(ModrinthVersion) -> bool)
        -> Result<Option<(ModrinthMod, ModrinthVersion, ModrinthVersionFile)>> {
        return if identifier.contains(':') { // User has specified a version ID, check that it actually exists for the specified mod.
            let vec: Vec<&str> = identifier.split(':').collect();
            if vec.len() != 2 {
                error(&*format!("Invalid version specifier: {} (too many colons)", identifier));
                Ok(None)
            } else {
                let mod_details = self.get_mod(&vec[0].to_string()).await?;
                let version_id = vec[1].to_string();
                return if mod_details.versions.contains(&version_id) {
                    let version = self.get_version(&version_id).await?;
                    Ok(version.resolve_file().map(|f| (mod_details, version, f)))
                } else {
                    error(&*format!("Invalid version for mod {}: {}", mod_details.title, vec[1]));
                    Ok(None)
                }
            }
        } else { // We need to figure out what version is the latest, as we just have a mod ID/slug
            let details = self.get_mod(identifier).await?;
            let versions = self.get_mod_versions(&details.id).await?;

            let mut filtered_versions = versions.iter().filter(|&v| predicate(v.clone())).collect::<Vec<&ModrinthVersion>>();

            filtered_versions.sort_by(|&v1, &v2| v1.date_published.cmp(&v2.date_published));

            return match filtered_versions.last() {
                None => Ok(None),
                Some(&version) => Ok(version.resolve_file().map(|f| (details, version.clone(), f)))
            }
        }
    }

    fn get_api_base(&self) -> &str {
        if self.staging {
            MODRINTH_STAGING_API
        } else {
            MODRINTH_API
        }
    }

    async fn get_mod(&self, slug: &str) -> Result<ModrinthMod> {
        Ok(self.client.get(format!("{}/mod/{}", self.get_api_base(), slug))
            .send().await?
            .json().await?)
    }

    async fn get_mod_versions(&self, mod_id: &str) -> Result<Vec<ModrinthVersion>> {
        Ok(self.client.get(format!("{}/mod/{}/version", self.get_api_base(), mod_id))
            .send().await?
            .json().await?)
    }

    async fn get_version(&self, version_id: &str) -> Result<ModrinthVersion> {
        Ok(self.client.get(format!("{}/version/{}", self.get_api_base(), version_id))
            .send().await?
            .json().await?)
    }
}
