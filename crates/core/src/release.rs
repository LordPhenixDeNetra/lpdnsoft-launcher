//! Releases publiées sur GitHub : étiquette, date, fichiers joints.

use crate::CHECKSUMS_ASSET;
use crate::app::{AppSpec, AssetPattern, Platform};
use crate::version::{Version, VersionError};

/// Jour de publication, tel qu'affiché dans la liste.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl Date {
    /// Lit le début d'une date ISO 8601 (`2026-10-28T12:00:00Z`) ; le reste est ignoré.
    pub fn parse_iso(text: &str) -> Option<Self> {
        let mut parts = text.split(['T', ' ']).next()?.split('-');
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        (1..=12).contains(&month).then_some(())?;
        (1..=31).contains(&day).then_some(())?;
        Some(Self { year, month, day })
    }

    /// `28/10/2026`, sans dépendre de la langue.
    pub fn short(self) -> String {
        format!("{:02}/{:02}/{}", self.day, self.month, self.year)
    }
}

/// Un fichier joint à une release.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

/// Une release publiée d'une application.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Release {
    pub tag: String,
    pub version: Version,
    pub published: Option<Date>,
    pub assets: Vec<Asset>,
    /// Page des notes de version.
    pub notes_url: String,
}

impl Release {
    pub fn new(
        tag: &str,
        published: Option<Date>,
        assets: Vec<Asset>,
        notes_url: impl Into<String>,
    ) -> Result<Self, VersionError> {
        Ok(Self {
            tag: tag.to_owned(),
            version: tag.parse()?,
            published,
            assets,
            notes_url: notes_url.into(),
        })
    }

    /// Release d'une application, construite à partir de son étiquette.
    pub fn for_app(
        app: &AppSpec,
        tag: &str,
        published: Option<Date>,
        assets: Vec<Asset>,
    ) -> Result<Self, VersionError> {
        Self::new(tag, published, assets, app.notes_url())
    }

    /// Paquet de cette plateforme, s'il est joint à la release.
    pub fn asset_for(&self, pattern: &AssetPattern, platform: Platform) -> Option<&Asset> {
        let expected = pattern.render(&self.version, platform);
        self.assets.iter().find(|asset| asset.name == expected)
    }

    /// Fichier d'empreintes (`SHA256SUMS`) joint à la release, s'il existe.
    pub fn checksums(&self) -> Option<&Asset> {
        self.assets
            .iter()
            .find(|asset| asset.name == CHECKSUMS_ASSET)
    }

    /// Taille lisible du paquet de cette plateforme (`84 Mo`).
    pub fn size_for(&self, pattern: &AssetPattern, platform: Platform) -> Option<u64> {
        self.asset_for(pattern, platform).map(|asset| asset.size)
    }
}

/// Taille en octets → `« 84 Mo »` ; `None` sous 1 Ko (illisible autrement).
pub fn size_text(bytes: u64) -> String {
    const KO: f64 = 1024.0;
    let bytes = bytes as f64;
    if bytes >= KO * KO * KO {
        format!("{:.1} Go", bytes / (KO * KO * KO))
    } else if bytes >= KO * KO {
        format!("{:.0} Mo", bytes / (KO * KO))
    } else if bytes >= KO {
        format!("{:.0} Ko", bytes / KO)
    } else {
        format!("{bytes:.0} o")
    }
}
