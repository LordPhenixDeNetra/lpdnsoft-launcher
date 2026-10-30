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
    match response.status().as_u16() {
        200 => {}
        404 => return Err(GithubError::NoRelease),
        403 | 429 => return Err(GithubError::RateLimited),
        code => return Err(GithubError::Network(format!("GitHub a répondu {code}"))),
    }
    let json: ReleaseJson = response
        .json()
        .await
        .map_err(|e| GithubError::Decode(e.to_string()))?;
    json.into_release(app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lpdnsoft_launcher_core::AssetPattern;

    fn app() -> AppSpec {
        AppSpec {
            id: "ankou".into(),
            name: "aNkou".into(),
            tagline: "VPN".into(),
            author: "LPDNSoft".into(),
            repo: "LordPhenixDeNetra/ankou".into(),
            homepage: None,
            asset: AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}"),
            hidden: false,
        }
    }

    #[test]
    fn maps_a_github_release() {
        let json: ReleaseJson = serde_json::from_str(
            r#"{
                "tag_name": "v1.0.1",
                "published_at": "2026-10-28T09:12:00Z",
                "html_url": "https://github.com/LordPhenixDeNetra/ankou/releases/tag/v1.0.1",
                "assets": [
                    {"name": "ankou-1.0.1-windows-x86_64.msi", "browser_download_url": "https://example.test/a.msi", "size": 4194304},
                    {"name": "SHA256SUMS", "browser_download_url": "https://example.test/SHA256SUMS", "size": 231}
                ]
            }"#,
        )
        .unwrap();
        let release = json.into_release(&app()).unwrap();
        assert_eq!(release.version.to_string(), "1.0.1");
        assert_eq!(release.published.unwrap().short(), "28/10/2026");
        assert_eq!(release.assets.len(), 2);
        assert_eq!(release.checksums().unwrap().name, "SHA256SUMS");
        assert!(release.notes_url.ends_with("/releases/tag/v1.0.1"));
    }

    #[test]
    fn refuses_a_tag_that_is_not_a_version() {
        let json: ReleaseJson = serde_json::from_str(r#"{"tag_name": "nightly"}"#).unwrap();
        assert!(matches!(
            json.into_release(&app()),
            Err(GithubError::Version(_))
        ));
    }
}
