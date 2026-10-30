//! Applications de la suite et plateformes : ce qui est publié, et sous quel nom de fichier.

use crate::version::Version;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fmt;

/// Système d'exploitation visé par un paquet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Os {
    Windows,
    Linux,
    Macos,
}

impl Os {
    /// Fragment utilisé dans les noms d'assets (`ankou-1.0.1-windows-x86_64.msi`).
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Macos => "macos",
        }
    }

    /// Extension du paquet installable (ADR 0011 d'aNkou).
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Windows => "msi",
            Self::Linux => "deb",
            Self::Macos => "pkg",
        }
    }

    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Linux
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Arch {
    X86_64,
    Aarch64,
}

impl Arch {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
        }
    }

    pub const fn current() -> Self {
        if cfg!(target_arch = "aarch64") {
            Self::Aarch64
        } else {
            Self::X86_64
        }
    }
}

/// Système et architecture de la machine qui fait tourner le launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    pub const fn new(os: Os, arch: Arch) -> Self {
        Self { os, arch }
    }

    /// Plateforme de la machine courante.
    pub const fn current() -> Self {
        Self::new(Os::current(), Arch::current())
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.os.slug(), self.arch.slug())
    }
}

/// Motif du fichier à télécharger, avec `{version}`, `{os}`, `{arch}` et `{ext}`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct AssetPattern(String);

impl AssetPattern {
    pub fn new(template: impl Into<String>) -> Self {
        Self(template.into())
    }

    /// `ankou-{version}-{os}-{arch}.{ext}` → `ankou-1.0.1-windows-x86_64.msi`.
    pub fn render(&self, version: &Version, platform: Platform) -> String {
        let version = version.to_string();
        self.0
            .replace("{version}", &version)
            .replace("{os}", platform.os.slug())
            .replace("{arch}", platform.arch.slug())
            .replace("{ext}", platform.os.extension())
    }
}

/// Une application de la suite, telle que décrite dans `apps.json`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AppSpec {
    /// Identifiant stable, en minuscules (`ankou`).
    pub id: String,
    pub name: String,
    /// Sous-titre affiché sous le nom.
    pub tagline: String,
    pub author: String,
    /// Dépôt GitHub `propriétaire/nom`.
    pub repo: String,
    #[serde(default)]
    pub homepage: Option<String>,
    /// Motif du paquet à télécharger pour cette application.
    pub asset: AssetPattern,
    /// Masquée dans la liste, sans être retirée du manifeste.
    #[serde(default)]
    pub hidden: bool,
}

impl AppSpec {
    pub fn repo_url(&self) -> String {
        format!("https://github.com/{}", self.repo)
    }

    pub fn notes_url(&self) -> String {
        format!("{}/releases", self.repo_url())
    }

    /// Nom du paquet de cette version pour cette plateforme.
    pub fn asset_name(&self, version: &Version, platform: Platform) -> String {
        self.asset.render(version, platform)
    }
}

/// Contenu de `apps.json` : la liste des applications de la suite.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Manifest {
    pub apps: Vec<AppSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ManifestError {
    #[error("manifeste illisible : {0}")]
    Json(String),
    #[error("identifiant en double : {0}")]
    DuplicateId(String),
    #[error("dépôt invalide pour {0} : attendu « propriétaire/nom »")]
    InvalidRepo(String),
}

impl Manifest {
    /// Lit et vérifie un manifeste.
    pub fn parse(json: &str) -> Result<Self, ManifestError> {
        let manifest: Self =
            serde_json::from_str(json).map_err(|e| ManifestError::Json(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Vérifie ce qu'un manifeste mal écrit pourrait casser : identifiants uniques,
    /// dépôts exploitables pour construire une URL.
    pub fn validate(&self) -> Result<(), ManifestError> {
        let mut ids = BTreeSet::new();
        for app in &self.apps {
            if !ids.insert(app.id.as_str()) {
                return Err(ManifestError::DuplicateId(app.id.clone()));
            }
            let repo = app.repo.split('/').collect::<Vec<_>>();
            let usable = repo.len() == 2 && repo.iter().all(|part| !part.is_empty());
            if !usable {
                return Err(ManifestError::InvalidRepo(app.id.clone()));
            }
        }
        Ok(())
    }

    /// Applications visibles, dans l'ordre du manifeste.
    pub fn visible(&self) -> impl Iterator<Item = &AppSpec> {
        self.apps.iter().filter(|app| !app.hidden)
    }
}
