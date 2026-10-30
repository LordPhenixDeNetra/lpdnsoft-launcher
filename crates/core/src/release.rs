//! Releases publiées sur GitHub : étiquette, date, fichiers joints.

use crate::CHECKSUMS_ASSET;
use crate::app::{AppSpec, AssetPattern, Platform};
use crate::version::{Version, VersionError};

/// Jour de publication, tel qu'affiché dans la liste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

/// Une release publiée d'une application.
#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Arch, Os};

    fn release() -> Release {
        let assets = vec![
            Asset {
                name: "ankou-1.0.1-windows-x86_64.msi".into(),
                url: "https://example.test/ankou.msi".into(),
                size: 84 * 1024 * 1024,
            },
            Asset {
                name: "SHA256SUMS".into(),
                url: "https://example.test/SHA256SUMS".into(),
                size: 120,
            },
        ];
        Release::new(
            "v1.0.1",
            Some(Date::parse_iso("2026-10-28T09:00:00Z").unwrap()),
            assets,
            "https://example.test",
        )
        .unwrap()
    }

    #[test]
    fn finds_the_package_of_a_platform() {
        let pattern = AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}");
        let windows = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(
            release().asset_for(&pattern, windows).unwrap().name,
            "ankou-1.0.1-windows-x86_64.msi"
        );
        let linux = Platform::new(Os::Linux, Arch::X86_64);
        assert!(release().asset_for(&pattern, linux).is_none());
    }

    #[test]
    fn exposes_checksums_and_size() {
        let pattern = AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}");
        let win = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(release().checksums().unwrap().name, "SHA256SUMS");
        assert_eq!(release().size_for(&pattern, win), Some(84 * 1024 * 1024));
        assert_eq!(size_text(84 * 1024 * 1024), "84 Mo");
    }

    #[test]
    fn reads_dates_and_rejects_nonsense() {
        let date = Date::parse_iso("2026-10-28T12:34:56Z").unwrap();
        assert_eq!(date.short(), "28/10/2026");
        assert!(Date::parse_iso("2026-13-01").is_none());
        assert!(Date::parse_iso("2026-10-32").is_none());
        assert!(Date::parse_iso("demain").is_none());
    }

    #[test]
    fn refuses_a_tag_that_is_not_a_version() {
        assert!(Release::new("nightly", None, Vec::new(), "u").is_err());
    }
}
