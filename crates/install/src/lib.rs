//! Installation d'un paquet téléchargé (ADR 0005).
//!
//! Trois étages, séparés pour être testables :
//!
//! 1. `plan` : la commande de base de l'installeur du système, sans élévation ;
//! 2. `elevated` : la même, enveloppée dans le mécanisme d'élévation du système ;
//! 3. `Executor` : l'exécution, réduite à un lancement de processus.
//!
//! Rien n'est deviné : une extension qui ne correspond pas au système est refusée, et un
//! chemin contenant un caractère dangereux pour une ligne de commande aussi.

use lpdnsoft_launcher_core::Os;
use std::path::Path;

/// Caractères qui n'ont rien à faire dans un chemin passé à PowerShell ou à AppleScript.
/// Ils sont refusés, jamais échappés : le chemin est construit par le launcher, un refus est
/// un bug visible, pas une perte.
const DANGEROUS: [char; 9] = ['"', '\'', '`', '$', ';', '|', '&', '\n', '\r'];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InstallError {
    #[error("format de paquet non pris en charge : {0}")]
    Unsupported(String),
    #[error("chemin refusé pour la ligne de commande : {0}")]
    Refused(String),
    #[error("l'installateur a échoué (code {code})")]
    Failed { code: i32 },
    #[error("exécution impossible : {0}")]
    Unavailable(String),
}

/// Ce qu'il faudrait lancer pour installer ce paquet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub program: String,
    pub args: Vec<String>,
    /// Vrai quand la commande passe par le mécanisme d'élévation du système.
    pub elevated: bool,
    /// Phrase courte, pour l'interface : ce que l'utilisateur s'apprête à autoriser.
    pub summary: String,
}

/// Commande de base de l'installeur, sans élévation.
pub fn plan(package: &Path, os: Os) -> Result<Plan, InstallError> {
    let path = package
        .to_str()
        .ok_or_else(|| InstallError::Refused(package.display().to_string()))?;
    if path.chars().any(|c| DANGEROUS.contains(&c)) {
        return Err(InstallError::Refused(path.to_owned()));
    }
    let extension = package
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let base = match (os, extension.as_str()) {
        (Os::Windows, "msi") => Plan {
            program: "msiexec".into(),
            // `/norestart` : le launcher ne redémarre jamais la machine à la place de
            // l'utilisateur ; `/qn` : pas d'interface, l'invite UAC est le consentement.
            args: vec!["/i".into(), path.into(), "/qn".into(), "/norestart".into()],
            elevated: false,
            summary: format!("Installer {path}"),
        },
        (Os::Linux, "deb") => Plan {
            program: "dpkg".into(),
            args: vec!["-i".into(), path.into()],
            elevated: false,
            summary: format!("Installer {path}"),
        },
        (Os::Linux, "rpm") => Plan {
            program: "rpm".into(),
            args: vec!["-U".into(), path.into()],
            elevated: false,
            summary: format!("Installer {path}"),
        },
        (Os::Macos, "pkg") => Plan {
            program: "installer".into(),
            args: vec!["-pkg".into(), path.into(), "-target".into(), "/".into()],
            elevated: false,
            summary: format!("Installer {path}"),
        },
        (_, extension) => {
            return Err(InstallError::Unsupported(extension.to_owned()));
        }
    };
    Ok(base)
}

/// Enveloppe le plan dans le mécanisme d'élévation du système.
///
/// Un plan déjà élevé est rendu tel quel : pas d'enveloppe dans l'enveloppe.
pub fn elevated(plan: Plan, os: Os) -> Plan {
    if plan.elevated {
        return plan;
    }
    let summary = format!("{} (autorisation administrateur)", plan.summary);
    match os {
        Os::Linux => Plan {
            program: "pkexec".into(),
            args: std::iter::once(plan.program).chain(plan.args).collect(),
            elevated: true,
            summary,
        },
        Os::Windows => {
            // `-Verb RunAs` déclenche UAC. Les arguments sont entre guillemets simples :
            // un `'` dans le chemin a déjà été refusé, donc rien ne peut s'échapper.
            let arguments = std::iter::once(plan.program.clone())
                .chain(plan.args.iter().cloned())
                .map(|argument| format!("'{argument}'"))
                .collect::<Vec<_>>()
                .join(",");
            Plan {
                program: "powershell".into(),
                args: vec![
                    "-NoProfile".into(),
                    "-NonInteractive".into(),
                    "-Command".into(),
                    format!(
                        "Start-Process -FilePath '{0}' -ArgumentList {arguments} -Verb RunAs -Wait",
                        plan.program
                    ),
                ],
                elevated: true,
                summary,
            }
        }
        Os::Macos => {
            let command = std::iter::once(plan.program.clone())
                .chain(plan.args.iter().cloned())
                .map(|argument| format!("'{argument}'"))
                .collect::<Vec<_>>()
                .join(" ");
            Plan {
                program: "osascript".into(),
                args: vec![
                    "-e".into(),
                    format!("do shell script \"{command}\" with administrator privileges"),
                ],
                elevated: true,
                summary,
            }
        }
    }
}

/// Exécution d'un plan.
pub trait Executor {
    fn run(&self, plan: &Plan) -> Result<(), InstallError>;
}

/// Exécutant réel : lance la commande du plan et relaie son code de sortie.
pub struct SystemExecutor;

impl Executor for SystemExecutor {
    fn run(&self, plan: &Plan) -> Result<(), InstallError> {
        let status = std::process::Command::new(&plan.program)
            .args(&plan.args)
            .status()
            .map_err(|e| InstallError::Unavailable(e.to_string()))?;
        if status.success() {
            Ok(())
        } else {
            Err(InstallError::Failed {
                code: status.code().unwrap_or(-1),
            })
        }
    }
}
