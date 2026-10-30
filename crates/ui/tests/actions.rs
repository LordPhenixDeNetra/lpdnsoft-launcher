//! Tests du contrat public : quelles actions mènent quelque part, et vers quoi.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{AppSpec, AssetPattern, Packages};
use lpdnsoft_launcher_ui::links::{available, url_for};
use lpdnsoft_launcher_ui::screen::Request;

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

#[test]
fn the_repository_link_is_always_available() {
    let app = app(None);
    let request = Request::Repository("ankou".into());
    assert!(available(&app, &request));
    assert_eq!(
        url_for(&app, &request).unwrap(),
        "https://github.com/LordPhenixDeNetra/ankou"
    );
}

#[test]
fn the_website_needs_a_homepage() {
    let request = Request::Website("ankou".into());
    assert!(available(&app(Some("https://lpdnsoft.fr")), &request));
    assert!(!available(&app(None), &request));
    assert!(url_for(&app(None), &request).is_none());
}

#[test]
fn refresh_is_available_without_any_link() {
    assert!(available(&app(None), &Request::Refresh));
}

#[test]
fn nothing_else_is_available_yet() {
    let app = app(Some("https://lpdnsoft.fr"));
    let pending = [
        Request::UpdateAll,
        Request::Install("ankou".into()),
        Request::Update("ankou".into()),
        Request::Open("ankou".into()),
        Request::ChooseVersion("ankou".into()),
        Request::Notes("ankou".into()),
        Request::AllVersions("ankou".into()),
        Request::Hide("ankou".into()),
    ];
    for request in &pending {
        assert!(
            !available(&app, request),
            "{request:?} ne devrait pas être disponible"
        );
        assert!(
            url_for(&app, request).is_none(),
            "{request:?} ne devrait pas avoir d'adresse"
        );
    }
}
