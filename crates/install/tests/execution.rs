//! Tests du contrat public : l'exécution d'un plan.
//!
//! Aucune installation n'est lancée : on vérifie le classement des échecs, pas un installeur.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_install::{Executor, InstallError, Plan, SystemExecutor};

fn plan(program: &str, args: &[&str]) -> Plan {
    Plan {
        program: program.to_owned(),
        args: args.iter().map(|a| (*a).to_owned()).collect(),
        elevated: false,
        summary: String::new(),
    }
}

#[test]
fn reports_a_missing_program() {
    let error = SystemExecutor
        .run(&plan("lpdnsoft-launcher-programme-absent", &[]))
        .unwrap_err();
    assert!(matches!(error, InstallError::Unavailable(_)), "{error:?}");
}

#[cfg(unix)]
#[test]
fn reports_the_exit_code_of_a_failed_installer() {
    let error = SystemExecutor
        .run(&plan("sh", &["-c", "exit 3"]))
        .unwrap_err();
    assert_eq!(error, InstallError::Failed { code: 3 });
}

#[cfg(unix)]
#[test]
fn accepts_a_successful_installer() {
    assert!(SystemExecutor.run(&plan("sh", &["-c", "exit 0"])).is_ok());
}

#[cfg(windows)]
#[test]
fn reports_the_exit_code_of_a_failed_installer() {
    let error = SystemExecutor
        .run(&plan("cmd", &["/C", "exit 3"]))
        .unwrap_err();
    assert_eq!(error, InstallError::Failed { code: 3 });
}

#[cfg(windows)]
#[test]
fn accepts_a_successful_installer() {
    assert!(SystemExecutor.run(&plan("cmd", &["/C", "exit 0"])).is_ok());
}
