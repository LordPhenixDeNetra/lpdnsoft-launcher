//! Tests du contrat public : le cache disque des releases.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_cache::{Cache, Entry};
use lpdnsoft_launcher_core::{Asset, Date, Release, Version};

fn release() -> Release {
    Release {
        tag: "v1.0.1".into(),
        version: Version::new(1, 0, 1),
        published: Some(Date {
            year: 2026,
            month: 10,
            day: 28,
        }),
        assets: vec![Asset {
            name: "ankou-1.0.1-windows-x86_64.msi".into(),
            url: "https://example.test/a.msi".into(),
            size: 1024,
        }],
        notes_url: "https://example.test/notes".into(),
        notes: None,
    }
}

#[test]
fn keeps_a_release_and_its_etag() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().join("cache"));
    cache
        .store(
            "ankou",
            &Entry {
                etag: Some("W/\"abc\"".into()),
                release: release(),
            },
        )
        .unwrap();
    let entry = cache.load("ankou").unwrap();
    assert_eq!(entry.etag.as_deref(), Some("W/\"abc\""));
    assert_eq!(entry.release, release());
}

#[test]
fn creates_its_directory_on_the_first_write() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a").join("b");
    let cache = Cache::new(&nested);
    cache
        .store(
            "ankou",
            &Entry {
                etag: None,
                release: release(),
            },
        )
        .unwrap();
    assert!(nested.join("ankou.json").is_file());
}

#[test]
fn returns_nothing_for_an_unknown_application() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path());
    assert!(cache.load("inconnue").is_none());
}

#[test]
fn ignores_a_corrupted_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("ankou.json"), b"pas du JSON").unwrap();
    let cache = Cache::new(dir.path());
    assert!(
        cache.load("ankou").is_none(),
        "un fichier illisible est ignoré"
    );
}

#[test]
fn a_disabled_cache_writes_nothing_and_complains_about_nothing() {
    let cache = Cache::disabled();
    assert!(cache.load("ankou").is_none());
    assert!(
        cache
            .store(
                "ankou",
                &Entry {
                    etag: None,
                    release: release()
                }
            )
            .is_ok()
    );
}

#[test]
fn still_reads_a_file_written_before_the_notes_field_existed() {
    // Un cache écrit par la version 0.4.0 : le champ `notes` n'y est pas. Il doit rester
    // lisible, sinon une mise à jour du launcher jette le cache (ADR 0003).
    let dir = tempfile::tempdir().unwrap();
    let old = r#"{
        "etag": "W/\"abc\"",
        "release": {
            "tag": "v1.0.1",
            "version": {"major": 1, "minor": 0, "patch": 1},
            "published": {"year": 2026, "month": 10, "day": 28},
            "assets": [],
            "notes_url": "https://example.test/notes"
        }
    }"#;
    std::fs::write(dir.path().join("ankou.json"), old).unwrap();
    let cache = Cache::new(dir.path());
    let entry = cache.load("ankou").unwrap();
    assert_eq!(entry.release.version, Version::new(1, 0, 1));
    assert!(entry.release.notes.is_none());
}
