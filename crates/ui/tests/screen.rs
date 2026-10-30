//! Tests du contrat public : écran et états de vérification.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use lpdnsoft_launcher_core::{AppSpec, AssetPattern, Date, Packages, Release, Row, State, Version};
use lpdnsoft_launcher_ui::localizer;
use lpdnsoft_launcher_ui::screen::{Check, Screen, Update};

fn app(id: &str, name: &str) -> AppSpec {
    AppSpec {
        id: id.into(),
        name: name.into(),
        tagline: "Application de test".into(),
        author: "LPDNSoft".into(),
        repo: format!("LordPhenixDeNetra/{id}"),
        homepage: None,
        asset: AssetPattern::new(format!("{id}-{{version}}-{{os}}-{{arch}}.{{ext}}")),
        packages: Packages::default(),
        hidden: false,
    }
}

fn row(id: &str, name: &str, installed: Option<&str>, available: &str) -> Row {
    let app = app(id, name);
    let release = Release::new(
        available,
        Some(Date {
            year: 2026,
            month: 10,
            day: 28,
        }),
        Vec::new(),
        "https://example.test",
    )
    .unwrap();
    Row::new(app, installed.map(|v| v.parse().unwrap()), Some(release))
}

fn screen(rows: Vec<Row>) -> Screen {
    let mut screen = Screen::new(localizer(Some("fr")).unwrap());
    screen.apply(Update::Catalog {
        rows,
        cached: false,
        openable: Default::default(),
    });
    screen
}

#[test]
fn renders_a_row_from_real_types() {
    let mut screen = screen(vec![row("ankou", "aNkou", Some("1.0.0"), "v1.0.1")]);
    let harness = Harness::new_ui(move |ui| screen.show(ui));
    harness.get_by_label("aNkou");
}

#[test]
fn counts_pending_updates() {
    let screen = screen(vec![
        row("a", "A", Some("1.0.0"), "v1.0.1"),
        row("b", "B", Some("2.0.0"), "v2.0.0"),
        row("c", "C", None, "v1.0.0"),
    ]);
    assert_eq!(screen.catalog.update_count(), 1);
}

#[test]
fn records_the_check_state() {
    let mut screen = Screen::new(localizer(Some("fr")).unwrap());
    assert_eq!(screen.check, Check::Never);
    screen.apply(Update::Checking);
    assert_eq!(screen.check, Check::Running);
    screen.apply(Update::Catalog {
        rows: Vec::new(),
        cached: false,
        openable: Default::default(),
    });
    assert_eq!(screen.check, Check::Done);
    screen.apply(Update::Failed("réseau".into()));
    assert_eq!(screen.check, Check::Failed("réseau".into()));
}

#[test]
fn says_when_the_catalog_comes_from_the_cache() {
    let mut screen = Screen::new(localizer(Some("fr")).unwrap());
    screen.apply(Update::Catalog {
        rows: Vec::new(),
        cached: true,
        openable: Default::default(),
    });
    assert_eq!(screen.check, Check::DoneCached);
}

#[test]
fn keeps_an_installed_version_newer_than_the_release() {
    let screen = screen(vec![row("a", "A", Some("1.2.0"), "v1.0.1")]);
    assert!(matches!(
        screen.catalog.rows()[0].state(),
        State::UpToDate { .. }
    ));
    assert_eq!(
        screen.catalog.rows()[0].installed,
        Some(Version::new(1, 2, 0))
    );
}

#[test]
fn shows_the_notice_of_a_failed_action() {
    let mut screen = screen(Vec::new());
    screen.notice = Some("Navigateur indisponible : test".into());
    let harness = Harness::new_ui(move |ui| screen.show(ui));
    harness.get_by_label("Navigateur indisponible : test");
}

#[test]
fn a_new_check_clears_the_notice() {
    let mut screen = screen(Vec::new());
    screen.notice = Some("Navigateur indisponible : test".into());
    screen.apply(Update::Checking);
    assert!(screen.notice.is_none());
}
