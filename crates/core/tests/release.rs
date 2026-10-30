//! Tests du contrat public : releases, dates et tailles.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{Arch, Asset, AssetPattern, Date, Os, Platform, Release, size_text};

fn release() -> Release {
    let assets = vec![
        Asset {
            name: "ankou-1.0.1-windows-x86_64.msi".into(),
            url: "https://example.test/ankou.msi".into(),
            size: 84 * 1024 * 1024,
        },
        Asset {
            name: "SHA256SUMS".into(),
            url: "https://example.test/SHA256SUMS".into(),
            size: 120,
        },
    ];
    Release::new(
        "v1.0.1",
        Some(Date::parse_iso("2026-10-28T09:00:00Z").unwrap()),
        assets,
        "https://example.test",
    )
    .unwrap()
}

#[test]
fn finds_the_package_of_a_platform() {
    let pattern = AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}");
    let windows = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(
        release().asset_for(&pattern, windows).unwrap().name,
        "ankou-1.0.1-windows-x86_64.msi"
    );
    let linux = Platform::new(Os::Linux, Arch::X86_64);
    assert!(release().asset_for(&pattern, linux).is_none());
}

#[test]
fn exposes_checksums_and_size() {
    let pattern = AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}");
    let win = Platform::new(Os::Windows, Arch::X86_64);
    assert_eq!(release().checksums().unwrap().name, "SHA256SUMS");
    assert_eq!(release().size_for(&pattern, win), Some(84 * 1024 * 1024));
    assert_eq!(size_text(84 * 1024 * 1024), "84 Mo");
}

#[test]
fn reads_dates_and_rejects_nonsense() {
    let date = Date::parse_iso("2026-10-28T12:34:56Z").unwrap();
    assert_eq!(date.short(), "28/10/2026");
    assert!(Date::parse_iso("2026-13-01").is_none());
    assert!(Date::parse_iso("2026-10-32").is_none());
    assert!(Date::parse_iso("demain").is_none());
}

#[test]
fn refuses_a_tag_that_is_not_a_version() {
    assert!(Release::new("nightly", None, Vec::new(), "u").is_err());
}
