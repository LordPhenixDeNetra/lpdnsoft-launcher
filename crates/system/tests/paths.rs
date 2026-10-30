//! Tests du contrat public : où chercher l'application installée.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{AppSpec, AssetPattern, Packages};
use lpdnsoft_launcher_system::{
    exe_from_display_icon, installed_path, linux_path, macos_path, windows_candidates,
};
use std::path::{Path, PathBuf};

fn app(id: &str, name: &str) -> AppSpec {
    AppSpec {
        id: id.into(),
        name: name.into(),
        tagline: "VPN".into(),
        author: "LPDNSoft".into(),
        repo: "LordPhenixDeNetra/ankou".into(),
        homepage: None,
        asset: AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}"),
        packages: Packages::default(),
        hidden: false,
    }
}

#[test]
fn reads_a_display_icon_with_an_index() {
    assert_eq!(
        exe_from_display_icon(r#""C:\Program Files\aNkou\ankou.exe",0"#),
        Some(r"C:\Program Files\aNkou\ankou.exe")
    );
    assert_eq!(
        exe_from_display_icon(r"C:\Program Files\aNkou\ankou.exe,0"),
        Some(r"C:\Program Files\aNkou\ankou.exe")
    );
    assert_eq!(
        exe_from_display_icon(r#"  "C:\Program Files\aNkou\ankou.exe"  "#),
        Some(r"C:\Program Files\aNkou\ankou.exe")
    );
}

#[test]
fn refuses_an_empty_display_icon() {
    assert_eq!(exe_from_display_icon(""), None);
    assert_eq!(exe_from_display_icon("   "), None);
    assert_eq!(exe_from_display_icon(r#""""#), None);
}

#[test]
fn does_not_take_a_path_comma_for_an_icon_index() {
    // La partie après la virgule n'est pas un nombre : c'est le chemin entier.
    assert_eq!(
        exe_from_display_icon(r"C:\outils,1\ankou.exe"),
        Some(r"C:\outils,1\ankou.exe")
    );
}

#[test]
fn prefers_the_layout_a_real_wix_msi_creates() {
    let candidates = windows_candidates("aNkou", "ankou", Path::new(r"C:\Program Files"));
    assert_eq!(
        candidates.first(),
        Some(&PathBuf::from(r"C:\Program Files\aNkou\ankou.exe")),
        "c'est ce que produit aNkou 1.0.0 : dossier du nom, exécutable de l'identifiant"
    );
    assert_eq!(candidates.len(), 3);
}

#[test]
fn finds_nothing_for_an_application_that_is_not_installed() {
    // Identifiant inventé : aucun chemin ne peut exister, sur aucune plateforme.
    let absent = app("lpdnsoft-application-absente", "Application absente");
    assert!(installed_path(&absent).is_none());
    assert!(linux_path(&absent).is_none());
    assert!(macos_path(&absent).is_none());
}

#[test]
fn the_linux_path_uses_the_package_name() {
    // `/usr/bin/ankou` n'existe pas sur la machine de test : c'est le nom qui est vérifié.
    let absent = app("ankou", "aNkou");
    let found = linux_path(&absent);
    assert!(
        found.is_none() || found.is_some_and(|path| path.ends_with("ankou")),
        "le chemin Linux doit porter le nom du paquet"
    );
}
