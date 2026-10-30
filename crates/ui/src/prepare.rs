//! Préparation d'une installation : quels fichiers télécharger, et sous quel nom (ADR 0005).
//!
//! Pure : rien n'est téléchargé ici. La partie réseau et l'exécution viennent après, dans le
//! pont, et restent minces.

use lpdnsoft_launcher_core::{Platform, Row};

/// Ce qu'il faut télécharger pour installer cette ligne, sur cette plateforme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    /// Nom du paquet attendu dans `SHA256SUMS` et sur le disque.
    pub name: String,
    /// Adresse du paquet.
    pub package_url: String,
    /// Adresse du fichier d'empreintes de la release.
    pub checksums_url: String,
    /// Taille annoncée, quand elle est connue.
    pub size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PrepareError {
    #[error("aucune version publiée à installer")]
    NoRelease,
    #[error("aucun paquet pour cette plateforme dans la release {0}")]
    NoPackage(String),
    #[error("la release {0} ne publie pas de fichier SHA256SUMS")]
    NoChecksums(String),
}

/// Choisit le paquet et le fichier d'empreintes de cette ligne.
pub fn prepare(row: &Row, platform: Platform) -> Result<Prepared, PrepareError> {
    let release = row.available.as_ref().ok_or(PrepareError::NoRelease)?;
    let asset = release
        .asset_for(&row.app.asset, platform)
        .ok_or_else(|| PrepareError::NoPackage(release.tag.clone()))?;
    let checksums = release
        .checksums()
        .ok_or_else(|| PrepareError::NoChecksums(release.tag.clone()))?;
    Ok(Prepared {
        name: asset.name.clone(),
        package_url: asset.url.clone(),
        checksums_url: checksums.url.clone(),
        size: (asset.size > 0).then_some(asset.size),
    })
}
