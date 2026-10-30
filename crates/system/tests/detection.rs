//! Tests du contrat public : les séquences de détection, avec un faux exécutant.
//!
//! Aucune machine Linux ou macOS n'est nécessaire : ce qui est testé ici, c'est l'ordre des
//! tentatives et le choix de la réponse, pas le lancement du programme.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{AppSpec, AssetPattern, Packages};
use lpdnsoft_launcher_system::{Runner, detect_linux, detect_macos};
use std::cell::RefCell;

/// Exécutant qui répond ce qu'on lui dit et retient les appels reçus.
struct Fake {
    answers: Vec<(&'static str, Option<String>)>,
    calls: RefCell<Vec<String>>,
}

impl Fake {
    fn new(answers: Vec<(&'static str, Option<String>)>) -> Self {
        Self {
            answers,
            calls: RefCell::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl Runner for Fake {
    fn output(&self, program: &str, args: &[&str]) -> Option<String> {
        self.calls
            .borrow_mut()
            .push(format!("{program} {}", args.join(" ")));
        self.answers
            .iter()
            .find(|(name, _)| *name == program)
            .and_then(|(_, output)| output.clone())
    }
}

fn app() -> AppSpec {
    AppSpec {
        id: "ankou".into(),
        name: "aNkou".into(),
        tagline: "VPN".into(),
        author: "LPDNSoft".into(),
        repo: "LordPhenixDeNetra/ankou".into(),
        homepage: None,
        asset: AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}"),
        packages: Packages {
            deb: Some("ankou".into()),
            rpm: Some("ankou-rpm".into()),
            pkg: Some("fr.lpdnsoft.ankou".into()),
        },
        hidden: false,
    }
}

#[test]
fn reads_the_version_announced_by_dpkg() {
    let runner = Fake::new(vec![("dpkg-query", Some("1:1.0.1-2ubuntu3\n".into()))]);
    let version = detect_linux(&app(), &runner).unwrap();
    assert_eq!(version.to_string(), "1.0.1");
    assert_eq!(runner.calls().len(), 1, "rpm ne doit pas être interrogé");
    assert!(runner.calls()[0].contains("ankou"), "{:?}", runner.calls());
}

#[test]
fn falls_back_to_rpm_when_dpkg_does_not_know_the_package() {
    let runner = Fake::new(vec![("dpkg-query", None), ("rpm", Some("1.0.1\n".into()))]);
    let version = detect_linux(&app(), &runner).unwrap();
    assert_eq!(version.to_string(), "1.0.1");
    assert_eq!(runner.calls().len(), 2);
    assert!(runner.calls()[0].starts_with("dpkg-query"));
    assert!(runner.calls()[1].starts_with("rpm"));
    assert!(runner.calls()[1].contains("ankou-rpm"));
}

#[test]
fn falls_back_to_rpm_when_dpkg_answers_something_unreadable() {
    let runner = Fake::new(vec![
        ("dpkg-query", Some("pas une version\n".into())),
        ("rpm", Some("2.0.0\n".into())),
    ]);
    assert_eq!(detect_linux(&app(), &runner).unwrap().to_string(), "2.0.0");
}

#[test]
fn returns_nothing_when_no_package_manager_knows_it() {
    let runner = Fake::new(vec![("dpkg-query", None), ("rpm", None)]);
    assert!(detect_linux(&app(), &runner).is_none());
}

#[test]
fn reads_the_version_of_a_macos_package() {
    let runner = Fake::new(vec![(
        "pkgutil",
        Some("package-id: fr.lpdnsoft.ankou\nversion: 1.0.1\nvolume: /\n".into()),
    )]);
    let version = detect_macos(&app(), &runner).unwrap();
    assert_eq!(version.to_string(), "1.0.1");
    assert!(
        runner.calls()[0].contains("fr.lpdnsoft.ankou"),
        "l'identifiant du paquet vient du manifeste : {:?}",
        runner.calls()
    );
}

#[test]
fn returns_nothing_when_the_macos_package_is_absent() {
    let runner = Fake::new(vec![("pkgutil", None)]);
    assert!(detect_macos(&app(), &runner).is_none());
}

#[test]
fn prefers_the_explicit_package_names_over_the_identifier() {
    let app = app();
    assert_eq!(app.deb_package(), "ankou");
    assert_eq!(app.rpm_package(), "ankou-rpm");
    assert_eq!(app.pkg_id(), "fr.lpdnsoft.ankou");
}

#[test]
fn falls_back_on_the_identifier_when_no_package_name_is_given() {
    let mut app = app();
    app.packages = Packages::default();
    assert_eq!(app.deb_package(), "ankou");
    assert_eq!(app.rpm_package(), "ankou");
    assert_eq!(app.pkg_id(), "ankou");
}
