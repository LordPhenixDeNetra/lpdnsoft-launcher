//! Cœur du launcher LPDNSoft : catalogue des applications, versions et état installé.
//!
//! Aucune entrée/sortie, aucun accès réseau : ce crate décide *quoi* afficher et *quoi*
//! faire ; l'interface et les téléchargements viennent ensuite (ADR 0001).

mod app;
mod catalog;
mod release;
mod version;

pub use app::{AppSpec, Arch, AssetPattern, Manifest, ManifestError, Os, Packages, Platform};
pub use catalog::{Catalog, Row, State};
pub use release::{Asset, Date, Release, size_text};
pub use version::{Version, VersionError};

/// Fichier d'empreintes joint à chaque release (`ankou-…` + `SHA256SUMS`, ADR 0011 d'aNkou).
pub const CHECKSUMS_ASSET: &str = "SHA256SUMS";
