//! Tests du contrat public : quelles actions mènent quelque part, et vers quoi.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{AppSpec, AssetPattern, Date, Packages, Release, Row, Version};
use lpdnsoft_launcher_ui::links::{available, url_for};
use lpdnsoft_launcher_ui::screen::Request;
use std::path::Path;

fn app(homepage: Option<&str>) -> AppSpec {
    AppSpec {
        id: "ankou".into(),
        name: "aNkou".into(),
        tagline: "VPN".into(),
        author: "LPDNSoft".into(),
        repo: "LordPhenixDeNetra/ankou".into(),
        homepage: homepage.map(str::to_owned),
        asset: AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}"),
        packages: Packages::default(),
        hidden: false,
    }
}

fn release(tag: &str) -> Release {
    Release {
        tag: tag.into(),
        version: tag.trim_start_matches('v').parse().unwrap(),
        published: Some(Date {
            year: 2026,
            month: 10,
            day: 28,
        }),
        assets: Vec::new(),
        notes_url: "https://example.test/notes".into(),
    }
}

/// Ligne avec une page d'accueil, éventuellement installée, éventuellement publiée.
fn row(installed: Option<&str>, published: Option<&str>) -> Row {
    Row::new(
        app(Some("https://lpdnsoft.fr")),
        installed.map(|v| v.parse().unwrap()),
        published.map(release),
    )
}

#[test]
fn the_repository_link_is_always_available() {
    let row = row(None, None);
    let request = Request::Repository("ankou".into());
    assert!(available(&row, &request, None));
    assert_eq!(
        url_for(&row.app, &request).unwrap(),
        "https://github.com/LordPhenixDeNetra/ankou"
    );
}

#[test]
fn the_website_needs_a_homepage() {
    let request = Request::Website("ankou".into());
    let with = Row::new(app(Some("https://lpdnsoft.fr")), None, None);
    let without = Row::new(app(None), None, None);
    assert!(available(&with, &request, None));
    assert!(!available(&without, &request, None));
    assert!(url_for(&without.app, &request).is_none());
}

#[test]
fn refresh_is_available_without_any_link() {
    assert!(available(&row(None, None), &Request::Refresh, None));
}

#[test]
fn install_is_available_only_when_nothing_is_installed() {
    assert!(available(
        &row(None, Some("v1.0.1")),
        &Request::Install("ankou".into()),
        None
    ));
    assert!(!available(
        &row(Some("1.0.1"), Some("v1.0.1")),
        &Request::Install("ankou".into()),
        None
    ));
    assert!(!available(
        &row(Some("1.0.0"), Some("v1.0.1")),
        &Request::Install("ankou".into()),
        None
    ));
}

#[test]
fn install_needs_a_published_release() {
    let orphan = Row::new(app(None), None, None);
    assert!(!available(&orphan, &Request::Install("ankou".into()), None));
}

#[test]
fn update_is_available_only_when_a_newer_release_exists() {
    assert!(available(
        &row(Some("1.0.0"), Some("v1.0.1")),
        &Request::Update("ankou".into()),
        None
    ));
    assert!(!available(
        &row(Some("1.0.1"), Some("v1.0.1")),
        &Request::Update("ankou".into()),
        None
    ));
    assert!(!available(
        &row(None, Some("v1.0.1")),
        &Request::Update("ankou".into()),
        None
    ));
}

#[test]
fn open_is_available_only_when_the_executable_was_found() {
    let installed = row(Some("1.0.0"), Some("v1.0.1"));
    let request = Request::Open("ankou".into());
    let path = Path::new("C:/Program Files/aNkou/ankou.exe");
    assert!(available(&installed, &request, Some(path)));
    assert!(!available(&installed, &request, None));
    // Un programme n'a jamais d'adresse : il n'est pas ouvert par le navigateur.
    assert!(url_for(&installed.app, &request).is_none());
}

#[test]
fn nothing_else_is_available_yet() {
    let row = row(Some("1.0.0"), Some("v1.0.1"));
    let pending = [
        Request::UpdateAll,
        Request::ChooseVersion("ankou".into()),
        Request::Notes("ankou".into()),
        Request::AllVersions("ankou".into()),
        Request::Hide("ankou".into()),
    ];
    for request in &pending {
        assert!(
            !available(&row, request, None),
            "{request:?} ne devrait pas être disponible"
        );
        assert!(
            url_for(&row.app, request).is_none(),
            "{request:?} ne devrait pas avoir d'adresse"
        );
    }
}

#[test]
fn a_version_is_never_compared_without_both_sides() {
    // « Version inconnue » (installée, sans release) ne propose rien d'actionnable.
    let unknown = Row::new(app(None), Some(Version::new(1, 0, 0)), None);
    assert!(!available(
        &unknown,
        &Request::Install("ankou".into()),
        None
    ));
    assert!(!available(&unknown, &Request::Update("ankou".into()), None));
}
