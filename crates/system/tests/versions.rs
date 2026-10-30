//! Tests du contrat public : analyse des sorties des gestionnaires de paquets.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_system::{
    version_from_deb, version_from_display, version_from_pkgutil, version_from_rpm,
};

#[test]
fn removes_the_debian_epoch_and_revision() {
    assert_eq!(version_from_deb("1.0.1").unwrap().to_string(), "1.0.1");
    assert_eq!(
        version_from_deb("1:1.0.1-2ubuntu3").unwrap().to_string(),
        "1.0.1"
    );
    assert_eq!(
        version_from_deb("2:3.4.5+deb12").unwrap().to_string(),
        "3.4.5"
    );
}

#[test]
fn reads_an_rpm_version() {
    assert_eq!(version_from_rpm("1.0.1").unwrap().to_string(), "1.0.1");
    assert_eq!(
        version_from_rpm("  v2.10.0\n").unwrap().to_string(),
        "2.10.0"
    );
}

#[test]
fn refuses_a_prerelease() {
    // `~` marque une préversion : la traiter comme la version finale ferait croire à une
    // version que l'utilisateur n'a pas.
    assert!(version_from_deb("1.0.1~rc1-1").is_none());
    assert!(version_from_rpm("2.0.0~beta1").is_none());
}

#[test]
fn refuses_what_has_not_three_numbers() {
    for text in ["", "abc", "1", "1.0", "nightly"] {
        assert!(version_from_deb(text).is_none(), "{text} accepté à tort");
        assert!(version_from_rpm(text).is_none(), "{text} accepté à tort");
    }
}

#[test]
fn reads_the_version_line_of_pkgutil() {
    let output = "\
package-id: fr.lpdnsoft.ankou
version: 1.0.1
volume: /
install-time: 1698600000
";
    assert_eq!(version_from_pkgutil(output).unwrap().to_string(), "1.0.1");
}

#[test]
fn refuses_a_pkgutil_output_without_version() {
    let output = "package-id: fr.lpdnsoft.ankou\nvolume: /\n";
    assert!(version_from_pkgutil(output).is_none());
}

#[test]
fn keeps_reading_display_versions_from_the_registry() {
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
    assert!(version_from_display("1.0").is_none());
}
