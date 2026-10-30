//! Version réellement installée sur le poste (ADR 0002).
//!
//! Windows : les clés de désinstallation du registre (celle de la machine et celle de
//! l'utilisateur), où l'installeur MSI inscrit son nom et sa version. Ailleurs : rien pour
//! l'instant, et la liste affichera « inconnue ».

use lpdnsoft_launcher_core::{AppSpec, Version};

/// Nombre de composants retenus d'une version affichée par un installeur.
const COMPONENTS: usize = 3;

/// Version installée de cette application, si elle est inscrite sur le poste.
pub fn installed_version(app: &AppSpec) -> Option<Version> {
    backend::installed(app)
}

/// `1.0.1`, `v1.0.1` et `1.0.1.0` désignent la même version : on garde les trois premiers
/// nombres. Tout le reste (texte libre, `1.0`) n'est pas exploitable.
pub fn version_from_display(text: &str) -> Option<Version> {
    let core = text
        .trim()
        .split([' ', '-', '+'])
        .next()
        .unwrap_or_default();
    let numbers: Vec<&str> = core.split('.').take(COMPONENTS).collect();
    if numbers.len() < COMPONENTS {
        return None;
    }
    format!("{}.{}.{}", numbers[0], numbers[1], numbers[2])
        .parse()
        .ok()
}

#[cfg(windows)]
mod backend {
    use super::version_from_display;
    use lpdnsoft_launcher_core::{AppSpec, Version};
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

    const UNINSTALL: [&str; 2] = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    pub fn installed(app: &AppSpec) -> Option<Version> {
        let needle = app.name.to_lowercase();
        for hive in [
            RegKey::predef(HKEY_LOCAL_MACHINE),
            RegKey::predef(HKEY_CURRENT_USER),
        ] {
            for path in UNINSTALL {
                let Ok(key) = hive.open_subkey_with_flags(path, KEY_READ) else {
                    continue;
                };
                for entry in key.enum_keys().flatten() {
                    let Ok(entry) = key.open_subkey_with_flags(&entry, KEY_READ) else {
                        continue;
                    };
                    let Ok(name) = entry.get_value::<String, _>("DisplayName") else {
                        continue;
                    };
                    if !name.to_lowercase().contains(&needle) {
                        continue;
                    }
                    let Ok(text) = entry.get_value::<String, _>("DisplayVersion") else {
                        continue;
                    };
                    if let Some(version) = version_from_display(&text) {
                        return Some(version);
                    }
                }
            }
        }
        None
    }
}

#[cfg(not(windows))]
mod backend {
    use lpdnsoft_launcher_core::{AppSpec, Version};

    /// Pas encore de détection hors Windows : mieux vaut « inconnue » qu'une valeur inventée.
    pub fn installed(_app: &AppSpec) -> Option<Version> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_shapes_installers_write() {
        assert_eq!(version_from_display("1.0.1").unwrap().to_string(), "1.0.1");
        assert_eq!(version_from_display("v1.0.1").unwrap().to_string(), "1.0.1");
        assert_eq!(
            version_from_display("1.0.1.0").unwrap().to_string(),
            "1.0.1"
        );
        assert_eq!(
            version_from_display("  2.10.3 (build 7)  ")
                .unwrap()
                .to_string(),
            "2.10.3"
        );
    }

    #[test]
    fn refuses_what_is_not_a_version() {
        for text in ["", "1.0", "1", "nightly", "1.x.0", "a.b.c"] {
            assert!(
                version_from_display(text).is_none(),
                "{text} accepté à tort"
            );
        }
    }
}
