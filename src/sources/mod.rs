pub mod curseforge;
pub mod modrinth;
pub mod github;

#[derive(thiserror::Error, Debug)]
pub enum ResolutionError {
    #[error("unknown slug: {0}")]
    UnknownSlug(String),
}
