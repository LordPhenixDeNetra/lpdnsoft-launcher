//! Quelles actions mènent quelque part, et vers quelle adresse.
//!
//! L'interface n'affiche jamais un bouton actif qui ne ferait rien : cette module décide
//! ce qui est disponible, et le reste est désactivé avec une infobulle (ADR 0002).

use crate::screen::Request;
use lpdnsoft_launcher_core::AppSpec;

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
/// Installer, mettre à jour, ouvrir l'application, choisir une version, voir les notes,
/// toutes les versions et masquer n'existent pas encore : elles restent désactivées.
pub fn available(app: &AppSpec, request: &Request) -> bool {
    matches!(request, Request::Refresh) || url_for(app, request).is_some()
}
