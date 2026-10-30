//! Interface du launcher LPDNSoft : catalogue réel, thème commun.

mod app;
pub mod bridge;
pub mod links;
mod open;
pub mod prepare;
pub mod screen;

pub use app::run;
use lpdnsoft_i18n::{LanguageIdentifier, Localizer, Resource, langid};
use lpdnsoft_launcher_core::Manifest;

/// Manifeste embarqué : la liste des applications de la suite (ADR 0002).
const APPS: &str = include_str!("../../../apps.json");

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("{0}")]
    Display(String),
    #[error("{0}")]
    I18n(#[from] lpdnsoft_i18n::I18nError),
    #[error("manifeste d'applications invalide : {0}")]
    Manifest(String),
}

/// Traductions embarquées ; `lang` (`"en"`, `"fr"`) sinon la langue du système, repli `en`.
pub fn localizer(lang: Option<&str>) -> Result<Localizer, UiError> {
    let mut loc = Localizer::new(
        vec![
            Resource {
                locale: langid!("en"),
                source: include_str!("../../../locales/en/launcher.ftl"),
            },
            Resource {
                locale: langid!("fr"),
                source: include_str!("../../../locales/fr/launcher.ftl"),
            },
        ],
        langid!("en"),
    )?;
    let requested: Vec<LanguageIdentifier> = match lang {
        Some(l) => l.parse().into_iter().collect(),
        None => lpdnsoft_i18n::system_locales(),
    };
    loc.negotiate(&requested);
    Ok(loc)
}

/// Manifeste des applications, vérifié avant usage.
pub fn manifest() -> Result<Manifest, UiError> {
    Manifest::parse(APPS).map_err(|e| UiError::Manifest(e.to_string()))
}
