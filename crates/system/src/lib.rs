//! Version réellement installée sur le poste (ADR 0002, spécification de détection).
//!
//! Un seul point d'entrée pour l'appelant (`installed_version`), mais trois stratégies :
//!
//! - **Windows** : les clés de désinstallation du registre, où l'installeur MSI s'inscrit ;
//! - **Linux** : `dpkg-query` puis `rpm -q`, sans shell ;
//! - **macOS** : `pkgutil --pkg-info`, sur l'identifiant inverse du paquet.
//!
//! Les analyseurs de sortie et les séquences d'appel sont purs vis-à-vis d'un trait
//! `Runner`, donc testables sur n'importe quelle machine.

use lpdnsoft_launcher_core::{AppSpec, Version};
use std::path::{Path, PathBuf};

/// Nombre de composants retenus d'une version affichée par un installateur.
const COMPONENTS: usize = 3;

/// Version installée de cette application, si elle est connue du système.
pub fn installed_version(app: &AppSpec) -> Option<Version> {
    #[cfg(target_os = "macos")]
    {
        detect_macos(app, &SystemRunner)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        detect_linux(app, &SystemRunner)
    }
    #[cfg(windows)]
    {
        windows::installed(app)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = app;
        None
    }
}

/// Exécutant : rend la sortie d'une commande, ou `None` si elle échoue ou n'existe pas.
///
/// Le trait existe pour que les séquences ci-dessous soient testables sans machine Linux
/// ni macOS, avec un faux exécutant qui répond ce qu'on veut.
pub trait Runner {
    fn output(&self, program: &str, args: &[&str]) -> Option<String>;
}

/// Exécutant réel : lance le programme directement, sans passer par un shell.
pub struct SystemRunner;

impl Runner for SystemRunner {
    fn output(&self, program: &str, args: &[&str]) -> Option<String> {
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8(output.stdout).ok()
    }
}

/// Détection Linux : `dpkg-query` d'abord, `rpm` ensuite.
///
/// L'ordre est fixe et documenté : sur une Debian où `rpm` est présent, c'est tout de même
/// dpkg qui fait foi.
pub fn detect_linux(app: &AppSpec, runner: &dyn Runner) -> Option<Version> {
    let deb = runner
        .output("dpkg-query", &["-W", "-f=${Version}\n", app.deb_package()])
        .and_then(|output| version_from_deb(&output));
    deb.or_else(|| {
        runner
            .output("rpm", &["-q", "--qf", "%{VERSION}\n", app.rpm_package()])
            .and_then(|output| version_from_rpm(&output))
    })
}

/// Détection macOS : `pkgutil --pkg-info <identifiant>`.
pub fn detect_macos(app: &AppSpec, runner: &dyn Runner) -> Option<Version> {
    runner
        .output("pkgutil", &["--pkg-info", app.pkg_id()])
        .and_then(|output| version_from_pkgutil(&output))
}

/// `1:1.0.1-2ubuntu3` → `1.0.1` : époque Debian et révision de paquet retirées.
pub fn version_from_deb(text: &str) -> Option<Version> {
    leading_version(text)
}

/// `1.0.1` → `1.0.1` (`rpm` ne donne que la version amont).
pub fn version_from_rpm(text: &str) -> Option<Version> {
    leading_version(text)
}

/// Sortie de `pkgutil --pkg-info` → la valeur de sa ligne `version:`.
pub fn version_from_pkgutil(output: &str) -> Option<Version> {
    output.lines().find_map(|line| {
        line.trim()
            .strip_prefix("version:")
            .map(str::trim)
            .and_then(leading_version)
    })
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

/// Première suite `nombre.nombre.nombre` d'un texte, après retrait de l'époque.
///
/// Un `~` juste après la suite marque une préversion (`1.0.1~rc1`) : elle est refusée, comme
/// partout ailleurs dans la suite. Une révision (`-2ubuntu3`) ou un suffixe de reconstruction
/// (`+deb12`) ne changent pas la version et sont ignorés.
fn leading_version(text: &str) -> Option<Version> {
    let text = strip_epoch(text.trim());
    let start = text.find(|c: char| c.is_ascii_digit())?;
    let rest = &text[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(rest.len());
    if rest[end..].starts_with('~') {
        return None;
    }
    let candidate = rest[..end].trim_end_matches('.');
    let mut parts = candidate.split('.').take(COMPONENTS);
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some(Version::new(major, minor, patch))
}

/// Retire une époque `1:` quand elle est bien numérique.
fn strip_epoch(text: &str) -> &str {
    match text.split_once(':') {
        Some((epoch, rest)) if !epoch.is_empty() && epoch.bytes().all(|b| b.is_ascii_digit()) => {
            rest
        }
        _ => text,
    }
}

/// Chemin de l'application installée, s'il existe réellement sur le disque.
pub fn installed_path(app: &AppSpec) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        windows::installed_path(app)
    }
    #[cfg(target_os = "macos")]
    {
        macos_path(app)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        linux_path(app)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = app;
        None
    }
}

/// `/usr/bin/<paquet>` — là où le `.deb` et le `.rpm` posent le binaire.
pub fn linux_path(app: &AppSpec) -> Option<PathBuf> {
    let candidate = Path::new("/usr/bin").join(app.deb_package());
    candidate.is_file().then_some(candidate)
}

/// `/Applications/<Nom>.app` — le paquet macOS installé par `installer`.
pub fn macos_path(app: &AppSpec) -> Option<PathBuf> {
    let candidate = Path::new("/Applications").join(format!("{}.app", app.name));
    candidate.is_dir().then_some(candidate)
}

/// Extrait le chemin d'une valeur `DisplayIcon`, qui peut porter un index d'icône.
///
/// `"C:\Program Files\aNkou\ankou.exe",0` → `C:\Program Files\aNkou\ankou.exe`
pub fn exe_from_display_icon(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let without_index = match trimmed.rsplit_once(',') {
        Some((path, index))
            if !index.trim().is_empty() && index.trim().bytes().all(|b| b.is_ascii_digit()) =>
        {
            path
        }
        _ => trimmed,
    };
    let unquoted = without_index.trim().trim_matches('"').trim();
    (!unquoted.is_empty()).then_some(unquoted)
}

/// Emplacements essayés sous Windows quand le registre ne donne pas de chemin utilisable.
///
/// Le premier est celui d'un MSI WiX réel : `%ProgramFiles%\<nom>\<id>.exe`.
pub fn windows_candidates(display_name: &str, id: &str, program_files: &Path) -> Vec<PathBuf> {
    vec![
        program_files.join(display_name).join(format!("{id}.exe")),
        program_files.join(id).join(format!("{id}.exe")),
        program_files
            .join(display_name)
            .join(format!("{display_name}.exe")),
    ]
}

#[cfg(windows)]
mod windows {
    use super::version_from_display;
    use lpdnsoft_launcher_core::{AppSpec, Version};
    use std::path::{Path, PathBuf};
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

    const UNINSTALL: [&str; 2] = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    /// Entrée de désinstallation qui correspond à cette application.
    fn find_entry(app: &AppSpec) -> Option<RegKey> {
        let needle = app.name.to_lowercase();
        for hive in [
            RegKey::predef(HKEY_LOCAL_MACHINE),
            RegKey::predef(HKEY_CURRENT_USER),
        ] {
            for path in UNINSTALL {
                let Ok(key) = hive.open_subkey_with_flags(path, KEY_READ) else {
                    continue;
                };
                for name in key.enum_keys().flatten() {
                    let Ok(entry) = key.open_subkey_with_flags(&name, KEY_READ) else {
                        continue;
                    };
                    let Ok(display) = entry.get_value::<String, _>("DisplayName") else {
                        continue;
                    };
                    if display.to_lowercase().contains(&needle) {
                        return Some(entry);
                    }
                }
            }
        }
        None
    }

    pub fn installed(app: &AppSpec) -> Option<Version> {
        let entry = find_entry(app)?;
        let text = entry.get_value::<String, _>("DisplayVersion").ok()?;
        version_from_display(&text)
    }

    /// Chemin de l'exécutable : `DisplayIcon`, puis `InstallLocation`, puis les conventions.
    ///
    /// Sur un MSI WiX réel (aNkou 1.0.0), `InstallLocation` est vide et `DisplayIcon` absent :
    /// seule la convention `%ProgramFiles%\<nom>\<id>.exe` répond. Les trois sont essayées.
    pub fn installed_path(app: &AppSpec) -> Option<PathBuf> {
        let entry = find_entry(app);
        if let Some(entry) = &entry {
            if let Ok(icon) = entry.get_value::<String, _>("DisplayIcon")
                && let Some(exe) = super::exe_from_display_icon(&icon)
            {
                let path = PathBuf::from(exe);
                if path.is_file() {
                    return Some(path);
                }
            }
            if let Ok(location) = entry.get_value::<String, _>("InstallLocation") {
                let location = location.trim();
                if !location.is_empty() {
                    let path = Path::new(location).join(format!("{}.exe", app.id));
                    if path.is_file() {
                        return Some(path);
                    }
                }
            }
        }
        let program_files = std::env::var_os("ProgramFiles")?;
        super::windows_candidates(&app.name, &app.id, Path::new(&program_files))
            .into_iter()
            .find(|candidate| candidate.is_file())
    }
}
