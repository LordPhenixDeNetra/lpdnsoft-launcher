//! Tests du contrat public : manifeste et motifs d'asset.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{Arch, AssetPattern, Manifest, ManifestError, Os, Platform};

const JSON: &str = r#"{
        "apps": [{
            "id": "ankou",
            "name": "aNkou",
            "tagline": "Client VPN",
            "author": "LPDNSoft",
            "repo": "LordPhenixDeNetra/ankou",
            "homepage": "https://lpdnsoft.fr",
            "asset": "ankou-{version}-{os}-{arch}.{ext}"
        }]
    }"#;

#[test]
fn renders_assets_per_platform() {
    let pattern = AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}");
    let version = "1.0.1".parse().unwrap();
    for (platform, expected) in [
        (
            Platform::new(Os::Windows, Arch::X86_64),
            "ankou-1.0.1-windows-x86_64.msi",
        ),
        (
            Platform::new(Os::Linux, Arch::Aarch64),
            "ankou-1.0.1-linux-aarch64.deb",
        ),
        (
            Platform::new(Os::Macos, Arch::Aarch64),
            "ankou-1.0.1-macos-aarch64.pkg",
        ),
    ] {
        assert_eq!(pattern.render(&version, platform), expected);
    }
}

#[test]
fn parses_a_manifest_and_its_urls() {
    let manifest = Manifest::parse(JSON).unwrap();
    let app = &manifest.apps[0];
    assert_eq!(app.id, "ankou");
    assert_eq!(app.repo_url(), "https://github.com/LordPhenixDeNetra/ankou");
    assert_eq!(
        app.notes_url(),
        "https://github.com/LordPhenixDeNetra/ankou/releases"
    );
}

#[test]
fn rejects_duplicate_ids() {
    let json = r#"{"apps": [
            {"id": "ankou", "name": "a", "tagline": "b", "author": "c", "repo": "x/y", "asset": "f"},
            {"id": "ankou", "name": "a", "tagline": "b", "author": "c", "repo": "x/y", "asset": "f"}
        ]}"#;
    assert_eq!(
        Manifest::parse(json).unwrap_err(),
        ManifestError::DuplicateId("ankou".into())
    );
}

#[test]
fn rejects_unusable_repositories() {
    let broken = JSON.replace("LordPhenixDeNetra/ankou", "ankou");
    assert_eq!(
        Manifest::parse(&broken).unwrap_err(),
        ManifestError::InvalidRepo("ankou".into())
    );
}

#[test]
fn hidden_apps_stay_listed_but_are_not_shown() {
    let hidden = JSON.replace("\"asset\"", "\"hidden\": true, \"asset\"");
    let manifest = Manifest::parse(&hidden).unwrap();
    assert_eq!(manifest.apps.len(), 1);
    assert_eq!(manifest.visible().count(), 0);
}
