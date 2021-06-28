use std::collections::HashMap;

use anyhow::Result;
use chrono::NaiveDateTime;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::sources::ResolutionError;
use crate::util::USER_AGENT;

const SLUG_QUERY: &str = "query get_by_slug($slug: String) {
  addons(slug: $slug) {
    authors {
      name
    }
    name
    summary
    slug
    id
    files {
      downloadUrl
      fileName
      gameVersion
      id
      displayName
      fileDate
    }
  }
}";

#[derive(Serialize)]
struct CurseforgeLookupGQLRequest {
    query: String,
    variables: HashMap<String, String>,
    #[serde(rename = "operationName")]
    operation_name: String,
}

#[derive(Deserialize, Debug)]
pub struct CurseforgeModQuery {
    pub data: CurseforgeModData,
}

#[derive(Deserialize, Debug)]
pub struct CurseforgeModData {
    pub addons: Vec<CurseforgeAddon>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CurseforgeAddon {
    pub authors: Vec<CurseforgeModAuthor>,
    pub name: String,
    pub slug: String,
    pub id: i32,
    pub summary: String,
    pub files: Vec<CurseforgeModFile>,
}

impl CurseforgeAddon {
    pub fn format_authors(&self) -> String {
        self.authors.iter().map(|a| a.name.clone())
            .collect::<Vec<String>>().join(", ")
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CurseforgeModAuthor {
    name: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeModFile {
    pub download_url: String,
    pub file_name: String,
    pub game_version: Vec<String>,
    pub id: i32,
    pub display_name: String,
    pub file_date: NaiveDateTime,
}

impl CurseforgeLookupGQLRequest {
    fn create_slug_lookup(slug: &str) -> Self {
        Self {
            query: SLUG_QUERY.to_string(),
            variables: make_map_pair("slug", slug.to_string()),
            operation_name: "get_by_slug".to_string(),
        }
    }
}

fn make_map_pair(a: &str, b: String) -> HashMap<String, String> {
    let mut mp = HashMap::with_capacity(1);
    mp.insert(a.to_string(), b);
    mp
}

pub struct CurseforgeClient {
    client: Client,
}

impl CurseforgeClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .connection_verbose(true)
                .user_agent(USER_AGENT)
                .build()?
        })
    }

    pub async fn find_mod_by_slug(&self, slug: &str) -> Result<CurseforgeAddon> {
        let request = self.client.post("https://curse.nikky.moe/graphql")
            .json(&CurseforgeLookupGQLRequest::create_slug_lookup(&slug))
            .build()?;
        let addons = self.client.execute(request).await?
            .json::<CurseforgeModQuery>().await?.data.addons;
        if addons.is_empty() {
            Err(ResolutionError::UnknownSlug(slug.to_string()).into())
        } else {
            Ok(addons.first().expect("is_empty returned false for an empty Vec!?").clone())
        }
    }
}
