//! Lecture d'une release publiée sur GitHub (ADR 0002).
//!
//! Une seule requête par application : `GET /repos/<propriétaire>/<nom>/releases/latest`.
//! Aucune authentification, donc soixante requêtes par heure au total — d'où l'usage d'un
//! cache `ETag` à venir, et un plafond volontaire sur le nombre d'applications sincère.

use lpdnsoft_launcher_core::{AppSpec, Asset, Date, Release, VersionError};
use serde::Deserialize;
use std::time::Duration;

const API: &str = "https://api.github.com";
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GithubError {
    #[error("réseau : {0}")]
    Network(String),
    #[error("aucune release publiée")]
    NoRelease,
    #[error("quota de l'API GitHub atteint (60 requêtes par heure)")]
    RateLimited,
    #[error("réponse illisible : {0}")]
    Decode(String),
    #[error("{0}")]
    Version(#[from] VersionError),
}

#[derive(Debug, Deserialize)]
struct ReleaseJson {
    tag_name: String,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<AssetJson>,
    #[serde(default)]
    html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AssetJson {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

impl ReleaseJson {
    fn into_release(self, app: &AppSpec) -> Result<Release, GithubError> {
        let published = self.published_at.as_deref().and_then(Date::parse_iso);
        let assets = self
            .assets
            .into_iter()
            .map(|asset| Asset {
                name: asset.name,
                url: asset.browser_download_url,
                size: asset.size,
            })
            .collect();
        let notes = self.html_url.unwrap_or_else(|| app.notes_url());
        Release::new(&self.tag_name, published, assets, notes).map_err(GithubError::Version)
    }
}

/// Client HTTP du launcher : agent explicite, exigé par l'API GitHub.
pub fn client() -> Result<reqwest::Client, GithubError> {
    reqwest::Client::builder()
        .timeout(TIMEOUT)
        .user_agent(concat!("lpdnsoft-launcher/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| GithubError::Network(e.to_string()))
}

/// Sort d'une réponse d'après son statut : seul 200 porte une release lisible.
pub fn classify(status: u16) -> Result<(), GithubError> {
    match status {
        200 => Ok(()),
        404 => Err(GithubError::NoRelease),
        403 | 429 => Err(GithubError::RateLimited),
        code => Err(GithubError::Network(format!("GitHub a répondu {code}"))),
    }
}

/// Lit le corps JSON d'une release et le convertit.
///
/// Public pour être testable sans réseau : c'est le contrat de lecture de l'API, et le
/// seul endroit où la forme des réponses GitHub est connue.
pub fn parse_release(json: &str, app: &AppSpec) -> Result<Release, GithubError> {
    let raw: ReleaseJson =
        serde_json::from_str(json).map_err(|e| GithubError::Decode(e.to_string()))?;
    raw.into_release(app)
}

/// Dernière release publiée d'une application.
pub async fn latest_release(
    client: &reqwest::Client,
    app: &AppSpec,
) -> Result<Release, GithubError> {
    let url = format!("{API}/repos/{}/releases/latest", app.repo);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| GithubError::Network(e.to_string()))?;
    classify(response.status().as_u16())?;
    let body = response
        .text()
        .await
        .map_err(|e| GithubError::Decode(e.to_string()))?;
    parse_release(&body, app)
}
