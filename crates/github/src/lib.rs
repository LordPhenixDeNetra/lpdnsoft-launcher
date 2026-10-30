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
/// En-tête qui porte l'empreinte d'une réponse.
const ETAG_HEADER: &str = "ETag";
/// En-tête qui redemande une réponse seulement si elle a changé.
const IF_NONE_MATCH: &str = "If-None-Match";

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
    /// Texte des notes de version, écrit par l'auteur.
    #[serde(default)]
    body: Option<String>,
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
        Release::new(&self.tag_name, published, assets, notes)
            .map(|release| release.with_notes(self.body))
            .map_err(GithubError::Version)
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

/// Ce qu'une réponse signifie pour la lecture d'une release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// 200 : le corps porte la release.
    New,
    /// 304 : la version en cache reste valable, il n'y a rien à lire.
    CacheStillValid,
}

/// Sort d'une réponse d'après son statut. 304 est une issue normale, pas une erreur :
/// c'est la réponse qui ne consomme pas le quota de soixante requêtes par heure.
pub fn classify(status: u16) -> Result<Outcome, GithubError> {
    match status {
        200 => Ok(Outcome::New),
        304 => Ok(Outcome::CacheStillValid),
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

/// Nombre de versions demandées quand on ouvre l'historique.
pub const HISTORY_LIMIT: usize = 30;

/// Lit une liste de releases ; un élément illisible est ignoré plutôt que de tout perdre.
pub fn parse_releases(json: &str, app: &AppSpec) -> Result<Vec<Release>, GithubError> {
    let raw: Vec<ReleaseJson> =
        serde_json::from_str(json).map_err(|e| GithubError::Decode(e.to_string()))?;
    Ok(raw
        .into_iter()
        .filter_map(|release| release.into_release(app).ok())
        .collect())
}

/// Historique des versions publiées, de la plus récente à la plus ancienne.
///
/// Une requête par appel : elle n'est faite qu'à l'ouverture de la fenêtre d'historique.
pub async fn releases(
    client: &reqwest::Client,
    app: &AppSpec,
) -> Result<Vec<Release>, GithubError> {
    let url = format!("{API}/repos/{}/releases?per_page={HISTORY_LIMIT}", app.repo);
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
    parse_releases(&body, app)
}

/// Réponse à une lecture : la release, ou l'annonce que le cache suffit.
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    /// `None` quand GitHub répond 304 : garder la release déjà en cache.
    pub release: Option<Release>,
    /// Empreinte à renvoyer la prochaine fois, si le serveur en donne une.
    pub etag: Option<String>,
}

/// Dernière release publiée d'une application.
///
/// `etag` est l'empreinte de la dernière réponse : quand elle est fournie, GitHub répond
/// 304 sans compter la requête dans le quota.
pub async fn latest_release(
    client: &reqwest::Client,
    app: &AppSpec,
    etag: Option<&str>,
) -> Result<Answer, GithubError> {
    let url = format!("{API}/repos/{}/releases/latest", app.repo);
    let mut request = client.get(&url);
    if let Some(etag) = etag {
        request = request.header(IF_NONE_MATCH, etag);
    }
    let response = request
        .send()
        .await
        .map_err(|e| GithubError::Network(e.to_string()))?;
    let etag = response
        .headers()
        .get(ETAG_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    match classify(response.status().as_u16())? {
        Outcome::CacheStillValid => Ok(Answer {
            release: None,
            etag,
        }),
        Outcome::New => {
            let body = response
                .text()
                .await
                .map_err(|e| GithubError::Decode(e.to_string()))?;
            Ok(Answer {
                release: Some(parse_release(&body, app)?),
                etag,
            })
        }
    }
}
