use anyhow::Result;
use octocrab::models::repos::Asset;
use crate::util::warning;

pub struct GithubClient { }

impl GithubClient {
    pub fn new(github_token: Option<String>) -> Self {
        if github_token.is_none() {
            warning("It is recommended to set the GITHUB_TOKEN environment variable to your Github Personal Access Token (PAT) to allow pack-it to have higher rate-limit allowances")
        }

        let mut octocrab_builder = octocrab::OctocrabBuilder::new();
        if let Some(github_token) = github_token {
            octocrab_builder = octocrab_builder.personal_token(github_token);
        }
        octocrab::initialise(octocrab_builder).expect("Failed to initialise GitHub API client!");

        Self {}
    }

    pub async fn resolve_mod(&self, owner: &str, repo: &str, tag: &str) -> Result<Option<Asset>> {
        let octocrab = octocrab::instance();
        let release = octocrab.repos(owner, repo)
            .releases()
            .get_by_tag(tag)
            .await?;

        for asset in release.assets {
            if asset.name.ends_with(".jar") && !(asset.name.contains("-dev") || asset.name.contains("-sources")) {
                return Ok(Some(asset))
            }
        }

        Ok(None)
    }
}

pub fn get_github_token() -> Option<String> {
    std::env::vars().find(|(name, _)| name == "GITHUB_TOKEN").map(|(_, value)| value)
}
