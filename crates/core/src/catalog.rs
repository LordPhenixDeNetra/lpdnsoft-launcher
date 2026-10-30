//! Catalogue : les applications à afficher, avec leur version installée et la dernière publiée.

use crate::app::AppSpec;
use crate::release::Release;
use crate::version::Version;

/// Une application et ce que le système sait d'elle.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub app: AppSpec,
    /// Version installée sur ce poste, si elle a pu être lue.
    pub installed: Option<Version>,
    /// Dernière release publiée, si elle a pu être récupérée.
    pub available: Option<Release>,
}

/// Ce que la liste affiche dans la colonne « État ».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Rien d'installé : le bouton installe.
    NotInstalled,
    UpToDate {
        version: Version,
    },
    UpdateAvailable {
        installed: Version,
        available: Version,
    },
    /// Installée, mais aucune release connue : rien à comparer.
    Unknown,
}

impl Row {
    pub fn new(app: AppSpec, installed: Option<Version>, available: Option<Release>) -> Self {
        Self {
            app,
            installed,
            available,
        }
    }

    pub fn state(&self) -> State {
        match (self.installed, &self.available) {
            (None, _) => State::NotInstalled,
            (Some(_), None) => State::Unknown,
            (Some(installed), Some(release)) if release.version > installed => {
                State::UpdateAvailable {
                    installed,
                    available: release.version,
                }
            }
            (Some(version), Some(_)) => State::UpToDate { version },
        }
    }

    pub fn is_update(&self) -> bool {
        matches!(self.state(), State::UpdateAvailable { .. })
    }
}

/// Toutes les applications connues, dans l'ordre du manifeste.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    rows: Vec<Row>,
}

impl Catalog {
    pub fn new(rows: Vec<Row>) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn get(&self, id: &str) -> Option<&Row> {
        self.rows.iter().find(|row| row.app.id == id)
    }

    /// Nombre de mises à jour en attente (le « TOUT METTRE À JOUR » de l'en-tête).
    pub fn update_count(&self) -> usize {
        self.rows.iter().filter(|row| row.is_update()).count()
    }

    /// Lignes correspondant à `query` (nom, identifiant ou sous-titre, sans casse).
    pub fn visible(&self, query: &str) -> Vec<&Row> {
        let needle = query.trim().to_lowercase();
        self.rows
            .iter()
            .filter(|row| {
                needle.is_empty()
                    || row.app.name.to_lowercase().contains(&needle)
                    || row.app.id.to_lowercase().contains(&needle)
                    || row.app.tagline.to_lowercase().contains(&needle)
            })
            .collect()
    }
}
