//! Tests du contrat public : ce qui sera téléchargé pour installer une ligne.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{
    AppSpec, Arch, Asset, AssetPattern, Catalog, Date, Os, Platform, Release, Row, Version,
};
use lpdnsoft_launcher_ui::prepare::{PrepareError, prepare};

fn app() -> AppSpec {
    AppSpec {
        id: "ankou".into(),
        name: "aNkou".into(),
        tagline: "VPN".into(),
        author: "LPDNSoft".into(),
        repo: "LordPhenixDeNetra/ankou".into(),
        homepage: None,
        asset: AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}"),
        packages: Default::default(),
        hidden: false,
    }
}

fn windows() -> Platform {
    Platform::new(Os::Windows, Arch::X86_64)
}

fn release(assets: Vec<Asset>) -> Release {
    Release {
        tag: "v1.0.1".into(),
        version: Version::new(1, 0, 1),
        published: Some(Date {
            year: 2026,
            month: 10,
            day: 28,
        }),
        assets,
        notes_url: "https://example.test/notes".into(),
        notes: None,
    }
}

fn asset(name: &str, url: &str, size: u64) -> Asset {
    Asset {
        name: name.into(),
        url: url.into(),
        size,
    }
}

fn row(assets: Vec<Asset>) -> Row {
    Row::new(app(), Some(Version::new(1, 0, 0)), Some(release(assets)))
}

#[test]
fn picks_the_package_and_the_checksums_of_the_platform() {
    let prepared = prepare(
        &row(vec![
            asset(
                "ankou-1.0.1-windows-x86_64.msi",
                "https://example.test/a.msi",
                1024,
            ),
            asset("SHA256SUMS", "https://example.test/SHA256SUMS", 230),
            asset(
                "ankou-1.0.1-linux-x86_64.deb",
                "https://example.test/a.deb",
                2048,
            ),
        ]),
        windows(),
    )
    .unwrap();

    assert_eq!(prepared.name, "ankou-1.0.1-windows-x86_64.msi");
    assert_eq!(prepared.package_url, "https://example.test/a.msi");
    assert_eq!(prepared.checksums_url, "https://example.test/SHA256SUMS");
    assert_eq!(prepared.size, Some(1024));
}

#[test]
fn ignores_a_package_of_another_platform() {
    let error = prepare(
        &row(vec![asset(
            "ankou-1.0.1-linux-x86_64.deb",
            "https://example.test/a.deb",
            2048,
        )]),
        windows(),
    )
    .unwrap_err();
    assert_eq!(error, PrepareError::NoPackage("v1.0.1".into()));
}

#[test]
fn refuses_a_release_without_checksums() {
    let error = prepare(
        &row(vec![asset(
            "ankou-1.0.1-windows-x86_64.msi",
            "https://example.test/a.msi",
            1024,
        )]),
        windows(),
    )
    .unwrap_err();
    assert_eq!(error, PrepareError::NoChecksums("v1.0.1".into()));
}

#[test]
fn refuses_a_line_without_a_release() {
    let row = Row::new(app(), None, None);
    assert_eq!(
        prepare(&row, windows()).unwrap_err(),
        PrepareError::NoRelease
    );
}

#[test]
fn does_not_claim_a_size_when_the_asset_does_not_give_one() {
    let prepared = prepare(
        &row(vec![
            asset(
                "ankou-1.0.1-windows-x86_64.msi",
                "https://example.test/a.msi",
                0,
            ),
            asset("SHA256SUMS", "https://example.test/SHA256SUMS", 230),
        ]),
        windows(),
    )
    .unwrap();
    assert_eq!(prepared.size, None, "zéro n'est pas une taille connue");
}

#[test]
fn a_catalog_built_from_rows_is_consistent() {
    // Garde-fou : la ligne telle que le catalogue la produit reste préparable.
    let catalog = Catalog::new(vec![row(vec![
        asset(
            "ankou-1.0.1-windows-x86_64.msi",
            "https://example.test/a.msi",
            1024,
        ),
        asset("SHA256SUMS", "https://example.test/SHA256SUMS", 230),
    ])]);
    assert!(matches!(
        catalog.rows()[0].state(),
        lpdnsoft_launcher_core::State::UpdateAvailable { .. }
    ));
    assert!(prepare(&catalog.rows()[0], windows()).is_ok());
}
