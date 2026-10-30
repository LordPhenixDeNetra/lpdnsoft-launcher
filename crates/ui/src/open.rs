//! Ouverture d'une adresse dans le navigateur du système.
//!
//! Seules les URL `https://` sont acceptées : l'application n'ouvre jamais un chemin local
//! ni un schéma qui pourrait être détourné par un manifeste mal écrit.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OpenError {
    #[error("adresse refusée : {0}")]
    Refused(String),
    #[error("navigateur indisponible : {0}")]
    Unavailable(String),
}

/// Ouvre `target` dans le navigateur par défaut, sans attendre sa fermeture.
pub fn url(target: &str) -> Result<(), OpenError> {
    if !target.starts_with("https://") {
        return Err(OpenError::Refused(target.to_owned()));
    }
    let spawn = |command: &str, args: &[&str]| -> Result<(), OpenError> {
        std::process::Command::new(command)
            .args(args)
            .spawn()
            .map(|_| ())
            .map_err(|e| OpenError::Unavailable(e.to_string()))
    };
    #[cfg(windows)]
    {
        // Le premier argument vide est le titre de fenêtre : sans lui, `start` prend
        // l'adresse pour un titre et n'ouvre rien.
        spawn("cmd", &["/C", "start", "", target])
    }
    #[cfg(target_os = "macos")]
    {
        spawn("open", &[target])
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        spawn("xdg-open", &[target])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_anything_but_https() {
        for target in [
            "http://example.test",
            "file:///etc/passwd",
            "example.test",
            "",
        ] {
            assert_eq!(url(target), Err(OpenError::Refused(target.to_owned())));
        }
    }
}
