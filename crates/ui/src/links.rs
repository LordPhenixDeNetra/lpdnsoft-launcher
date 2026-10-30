//! Quelles actions mènent quelque part, et vers quelle adresse.
//!
//! L'interface n'affiche jamais un bouton actif qui ne ferait rien : ce module décide ce qui
//! est disponible, et le reste est désactivé avec une infobulle (ADR 0002, ADR 0005).

use crate::screen::Request;
use lpdnsoft_launcher_core::{AppSpec, Row, State};
use std::path::Path;

/// Adresse à ouvrir pour cette requête, s'il y en a une.
pub fn url_for(app: &AppSpec, request: &Request) -> Option<String> {
    match request {
        Request::Website(_) => app.homepage.clone(),
        Request::Repository(_) => Some(app.repo_url()),
        _ => None,
    }
}

/// Vrai si l'action produit un effet aujourd'hui.
///
/// `ChooseVersion`, `Notes`, `AllVersions` et `Hide` n'existent pas encore : elles restent
/// désactivées. `Install` n'a de sens que si rien n'est installé et qu'une release est
/// publiée ; `Update` seulement quand une version plus récente existe ; `Open` seulement
/// quand l'exécutable a été trouvé sur le disque.
pub fn available(row: &Row, request: &Request, openable: Option<&Path>) -> bool {
    match request {
        Request::Refresh => true,
        Request::Install(_) => {
            matches!(row.state(), State::NotInstalled) && row.available.is_some()
        }
        Request::Update(_) => matches!(row.state(), State::UpdateAvailable { .. }),
        Request::Open(_) => openable.is_some(),
        // Les notes sont dans la release déjà chargée ; l'historique demande une requête,
        // faite seulement à l'ouverture de la fenêtre.
        Request::Notes(_) | Request::AllVersions(_) => row.available.is_some(),
        _ => url_for(&row.app, request).is_some(),
    }
}
