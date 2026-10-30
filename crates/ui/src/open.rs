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

/// Lance l'application installée, sans attendre sa fermeture.
///
/// Le chemin vient de `installed_path`, qui n'accepte qu'un fichier réellement présent : on
/// n'exécute jamais un chemin venu d'ailleurs.
pub fn program(path: &std::path::Path) -> Result<(), OpenError> {
    if !path.exists() {
        return Err(OpenError::Refused(path.display().to_string()));
    }
    #[cfg(target_os = "macos")]
    {
        // Un `.app` est un dossier : c'est `open` qui sait le lancer.
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| OpenError::Unavailable(e.to_string()))
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::process::Command::new(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| OpenError::Unavailable(e.to_string()))
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

    #[test]
    fn refuses_a_program_that_does_not_exist() {
        let missing = std::path::Path::new("lpdnsoft-application-absente.exe");
        assert!(matches!(program(missing), Err(OpenError::Refused(_))));
    }
}
