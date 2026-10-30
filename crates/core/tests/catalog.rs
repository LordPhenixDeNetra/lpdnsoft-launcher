//! Tests du contrat public : catalogue et états.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{
    AppSpec, AssetPattern, Catalog, Platform, Release, Row, State, Version,
};

fn app(id: &str, name: &str, tagline: &str) -> AppSpec {
    AppSpec {
        id: id.into(),
        name: name.into(),
        tagline: tagline.into(),
        author: "LPDNSoft".into(),
        repo: format!("LordPhenixDeNetra/{id}"),
        homepage: None,
        asset: AssetPattern::new(format!("{id}-{{version}}-{{os}}-{{arch}}.{{ext}}")),
        hidden: false,
    }
}

fn release(version: &str) -> Release {
    Release::new(version, None, Vec::new(), "https://example.test").unwrap()
}

fn v(text: &str) -> Version {
    text.parse().unwrap()
}

#[test]
fn reports_each_state() {
    let a = app("ankou", "aNkou", "VPN");
    assert_eq!(Row::new(a.clone(), None, None).state(), State::NotInstalled);
    assert_eq!(
        Row::new(a.clone(), Some(v("1.0.1")), Some(release("v1.0.1"))).state(),
        State::UpToDate {
            version: v("1.0.1")
        }
    );
    assert_eq!(
        Row::new(a.clone(), Some(v("1.0.0")), Some(release("v1.0.1"))).state(),
        State::UpdateAvailable {
            installed: v("1.0.0"),
            available: v("1.0.1")
        }
    );
    assert_eq!(
        Row::new(a.clone(), Some(v("1.0.1")), None).state(),
        State::Unknown
    );
    // Une version installée plus récente que la dernière release n'est pas une mise à jour.
    assert!(matches!(
        Row::new(a, Some(v("1.2.0")), Some(release("v1.0.1"))).state(),
        State::UpToDate { .. }
    ));
}

#[test]
fn counts_updates_for_the_header_button() {
    let catalog = Catalog::new(vec![
        Row::new(
            app("ankou", "aNkou", "VPN"),
            Some(v("1.0.0")),
            Some(release("v1.0.1")),
        ),
        Row::new(
            app("b", "B", "x"),
            Some(v("2.0.0")),
            Some(release("v2.0.0")),
        ),
        Row::new(app("c", "C", "y"), None, Some(release("v1.0.0"))),
    ]);
    assert_eq!(catalog.update_count(), 1);
}

#[test]
fn filters_on_name_id_and_tagline() {
    let catalog = Catalog::new(vec![
        Row::new(app("ankou", "aNkou", "Client VPN"), None, None),
        Row::new(
            app("demo", "Demo", "Application de démonstration"),
            None,
            None,
        ),
    ]);
    assert_eq!(catalog.visible("").len(), 2);
    assert_eq!(catalog.visible("  ").len(), 2);
    assert_eq!(catalog.visible("ANKOU").len(), 1);
    assert_eq!(catalog.visible("vpn").len(), 1);
    assert_eq!(catalog.visible("démonstration").len(), 1);
    assert_eq!(catalog.visible("rien").len(), 0);
}

#[test]
fn looks_up_by_id() {
    let catalog = Catalog::new(vec![Row::new(app("ankou", "aNkou", "VPN"), None, None)]);
    assert!(catalog.get("ankou").is_some());
    assert!(catalog.get("absent").is_none());
}

#[test]
fn asset_pattern_needs_the_platform_it_was_written_for() {
    // Garde-fou : le motif ne doit pas dépendre d'une casse implicite.
    let spec = app("ankou", "aNkou", "VPN");
    let name = spec.asset_name(&v("1.0.1"), Platform::current());
    assert!(name.starts_with("ankou-1.0.1-"));
}
