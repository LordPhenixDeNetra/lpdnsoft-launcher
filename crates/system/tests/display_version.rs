//! Tests du contrat public : lecture d'une version affichée.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_system::version_from_display;

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
