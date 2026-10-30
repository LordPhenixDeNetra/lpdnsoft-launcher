//! Tests du contrat public : versions.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::Version;

fn version(text: &str) -> Version {
    text.parse().unwrap()
}

#[test]
fn parses_plain_and_tagged_versions() {
    assert_eq!(version("1.0.1"), Version::new(1, 0, 1));
    assert_eq!(version("v1.0.1"), Version::new(1, 0, 1));
    assert_eq!(version(" 12.340.5 "), Version::new(12, 340, 5));
}

#[test]
fn rejects_what_is_not_a_release() {
    for text in [
        "",
        "1",
        "1.2",
        "1.2.3.4",
        "1.2.3-rc1",
        "nightly",
        "v",
        "1..2",
        "-1.0.0",
    ] {
        assert!(
            text.parse::<Version>().is_err(),
            "{text} aurait dû être refusé"
        );
    }
}

#[test]
fn orders_by_number_not_by_text() {
    assert!(version("1.10.0") > version("1.9.0"));
    assert!(version("2.0.0") > version("1.99.99"));
    assert_eq!(version("1.0.0"), version("v1.0.0"));
}

#[test]
fn displays_canonically() {
    assert_eq!(version("v01.2.3").to_string(), "1.2.3");
}
