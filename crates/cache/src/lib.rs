//! Cache disque des releases : la dernière réponse connue et son empreinte (ADR 0003).
//!
//! Un fichier par application, écrit atomiquement. Le cache n'est pas une source de
//! vérité : il permet une requête conditionnelle (304, gratuite pour le quota) et il évite
//! une liste vide quand le réseau manque.

use lpdnsoft_launcher_core::Release;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Nom du dossier de configuration de l'application.
const APP: &str = "lpdnsoft-launcher";

/// Ce qu'on garde d'une réponse.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// Empreinte de la réponse, à renvoyer dans `If-None-Match`.
    #[serde(default)]
    pub etag: Option<String>,
    pub release: Release,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CacheError {
    #[error("écriture du cache impossible : {0}")]
    Write(String),
}

/// Dossier de cache ; `None` quand aucun dossier de configuration n'existe.
#[derive(Debug, Clone, Default)]
pub struct Cache {
    dir: Option<PathBuf>,
}

impl Cache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: Some(dir.into()),
        }
    }

    /// Cache du poste : `<config>/lpdnsoft-launcher/cache`.
    pub fn system() -> Self {
        Self {
            dir: lpdnsoft_settings::config_dir(APP).map(|dir| dir.join("cache")),
        }
    }

    /// Cache qui n'écrit rien : utile quand aucun dossier de configuration n'est connu.
    pub fn disabled() -> Self {
        Self { dir: None }
    }

    fn path(&self, id: &str) -> Option<PathBuf> {
        self.dir.as_ref().map(|dir| dir.join(format!("{id}.json")))
    }

    /// Entrée en cache, si elle existe et reste lisible. Un fichier corrompu est ignoré :
    /// le pire qu'il puisse arriver est une requête complète.
    pub fn load(&self, id: &str) -> Option<Entry> {
        let path = self.path(id)?;
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Enregistre l'entrée. Le cache désactivé ne fait rien et ne se plaint pas.
    pub fn store(&self, id: &str, entry: &Entry) -> Result<(), CacheError> {
        let Some(path) = self.path(id) else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| CacheError::Write(e.to_string()))?;
        }
        let text = serde_json::to_string(entry).map_err(|e| CacheError::Write(e.to_string()))?;
        lpdnsoft_atomic::write_atomic(&path, text.as_bytes())
            .map_err(|e| CacheError::Write(e.to_string()))
    }
}
