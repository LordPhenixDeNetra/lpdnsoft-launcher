//! Tests du contrat public : quel plan pour quel paquet, et comment il est élevé.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::Os;
use lpdnsoft_launcher_install::{InstallError, elevated, plan};
use std::path::Path;

const MSI: &str = "C:/cache/ankou-1.0.1-windows-x86_64.msi";
const DEB: &str = "/cache/ankou-1.0.1-linux-x86_64.deb";
const RPM: &str = "/cache/ankou-1.0.1-linux-x86_64.rpm";
const PKG: &str = "/cache/ankou-1.0.1-macos-aarch64.pkg";

#[test]
fn plans_a_windows_msi() {
    let found = plan(Path::new(MSI), Os::Windows).unwrap();
    assert_eq!(found.program, "msiexec");
    assert!(found.args.contains(&MSI.to_owned()));
    assert!(found.args.contains(&"/qn".to_owned()));
    assert!(
        found.args.contains(&"/norestart".to_owned()),
        "le launcher ne redémarre jamais la machine"
    );
    assert!(!found.elevated);
}

#[test]
fn plans_a_debian_package() {
    let found = plan(Path::new(DEB), Os::Linux).unwrap();
    assert_eq!(found.program, "dpkg");
    assert_eq!(found.args, ["-i".to_owned(), DEB.to_owned()]);
}

#[test]
fn plans_an_rpm_package() {
    let found = plan(Path::new(RPM), Os::Linux).unwrap();
    assert_eq!(found.program, "rpm");
    assert_eq!(found.args, ["-U".to_owned(), RPM.to_owned()]);
}

#[test]
fn plans_a_macos_package() {
    let found = plan(Path::new(PKG), Os::Macos).unwrap();
    assert_eq!(found.program, "installer");
    assert_eq!(found.args, ["-pkg", PKG, "-target", "/"].map(str::to_owned));
}

#[test]
fn refuses_a_package_that_does_not_match_the_system() {
    let error = plan(Path::new(DEB), Os::Windows).unwrap_err();
    assert_eq!(error, InstallError::Unsupported("deb".into()));
    let error = plan(Path::new(MSI), Os::Macos).unwrap_err();
    assert_eq!(error, InstallError::Unsupported("msi".into()));
    let error = plan(Path::new("/cache/paquet.zip"), Os::Linux).unwrap_err();
    assert_eq!(error, InstallError::Unsupported("zip".into()));
}

#[test]
fn refuses_a_path_that_could_escape_a_command_line() {
    for dangerous in [
        "C:/cache/ankou';calc'.msi",
        "C:/cache/ankou\".msi",
        "C:/cache/ankou$(calc).msi",
        "C:/cache/ankou;calc.msi",
        "C:/cache/ankou\ncalc.msi",
        "C:/cache/ankou|calc.msi",
        "C:/cache/ankou&calc.msi",
    ] {
        let error = plan(Path::new(dangerous), Os::Windows).unwrap_err();
        assert!(
            matches!(error, InstallError::Refused(_)),
            "{dangerous} aurait dû être refusé, reçu {error:?}"
        );
    }
}

#[test]
fn wraps_a_linux_install_with_pkexec() {
    let full = elevated(plan(Path::new(DEB), Os::Linux).unwrap(), Os::Linux);
    assert_eq!(full.program, "pkexec");
    assert_eq!(full.args.first().map(String::as_str), Some("dpkg"));
    assert!(full.elevated);
    assert!(full.summary.contains("administrateur"), "{}", full.summary);
}

#[test]
fn wraps_a_windows_install_with_uac() {
    let full = elevated(plan(Path::new(MSI), Os::Windows).unwrap(), Os::Windows);
    assert_eq!(full.program, "powershell");
    let command = full.args.last().unwrap();
    assert!(command.contains("-Verb RunAs"), "{command}");
    assert!(command.contains("'msiexec'"), "{command}");
    assert!(command.contains(MSI), "{command}");
    assert!(full.elevated);
}

#[test]
fn wraps_a_macos_install_with_the_system_authorization() {
    let full = elevated(plan(Path::new(PKG), Os::Macos).unwrap(), Os::Macos);
    assert_eq!(full.program, "osascript");
    let script = full.args.last().unwrap();
    assert!(script.contains("administrator privileges"), "{script}");
    assert!(script.contains("installer"), "{script}");
    assert!(script.contains(PKG), "{script}");
}

#[test]
fn does_not_wrap_an_already_elevated_plan() {
    let once = elevated(plan(Path::new(DEB), Os::Linux).unwrap(), Os::Linux);
    let twice = elevated(once.clone(), Os::Linux);
    assert_eq!(once, twice);
}
