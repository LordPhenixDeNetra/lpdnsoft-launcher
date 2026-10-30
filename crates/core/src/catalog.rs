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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AssetPattern, Platform};

    fn app(id: &str, name: &str, tagline: &str) -> AppSpec {
        AppSpec {
            id: id.into(),
            name: name.into(),
            tagline: tagline.into(),
            author: "LPDNSoft".into(),
            repo: format!("LordPhenixDeNetra/{id}"),
            homepage: None,
            asset: AssetPattern::new(format!("{id}-{{version}}-{{os}}-{{arch}}.{{ext}}")),
            hidden: false,
        }
    }

    fn release(version: &str) -> Release {
        Release::new(version, None, Vec::new(), "https://example.test").unwrap()
    }

    fn v(text: &str) -> Version {
        text.parse().unwrap()
    }

    #[test]
    fn reports_each_state() {
        let a = app("ankou", "aNkou", "VPN");
        assert_eq!(Row::new(a.clone(), None, None).state(), State::NotInstalled);
        assert_eq!(
            Row::new(a.clone(), Some(v("1.0.1")), Some(release("v1.0.1"))).state(),
            State::UpToDate {
                version: v("1.0.1")
            }
        );
        assert_eq!(
            Row::new(a.clone(), Some(v("1.0.0")), Some(release("v1.0.1"))).state(),
            State::UpdateAvailable {
                installed: v("1.0.0"),
                available: v("1.0.1")
            }
        );
        assert_eq!(
            Row::new(a.clone(), Some(v("1.0.1")), None).state(),
            State::Unknown
        );
        // Une version installée plus récente que la dernière release n'est pas une mise à jour.
        assert!(matches!(
            Row::new(a, Some(v("1.2.0")), Some(release("v1.0.1"))).state(),
            State::UpToDate { .. }
        ));
    }

    #[test]
    fn counts_updates_for_the_header_button() {
        let catalog = Catalog::new(vec![
            Row::new(
                app("ankou", "aNkou", "VPN"),
                Some(v("1.0.0")),
                Some(release("v1.0.1")),
            ),
            Row::new(
                app("b", "B", "x"),
                Some(v("2.0.0")),
                Some(release("v2.0.0")),
            ),
            Row::new(app("c", "C", "y"), None, Some(release("v1.0.0"))),
        ]);
        assert_eq!(catalog.update_count(), 1);
    }

    #[test]
    fn filters_on_name_id_and_tagline() {
        let catalog = Catalog::new(vec![
            Row::new(app("ankou", "aNkou", "Client VPN"), None, None),
            Row::new(
                app("demo", "Demo", "Application de démonstration"),
                None,
                None,
            ),
        ]);
        assert_eq!(catalog.visible("").len(), 2);
        assert_eq!(catalog.visible("  ").len(), 2);
        assert_eq!(catalog.visible("ANKOU").len(), 1);
        assert_eq!(catalog.visible("vpn").len(), 1);
        assert_eq!(catalog.visible("démonstration").len(), 1);
        assert_eq!(catalog.visible("rien").len(), 0);
    }

    #[test]
    fn looks_up_by_id() {
        let catalog = Catalog::new(vec![Row::new(app("ankou", "aNkou", "VPN"), None, None)]);
        assert!(catalog.get("ankou").is_some());
        assert!(catalog.get("absent").is_none());
    }

    #[test]
    fn asset_pattern_needs_the_platform_it_was_written_for() {
        // Garde-fou : le motif ne doit pas dépendre d'une casse implicite.
        let spec = app("ankou", "aNkou", "VPN");
        let name = spec.asset_name(&v("1.0.1"), Platform::current());
        assert!(name.starts_with("ankou-1.0.1-"));
    }
}
