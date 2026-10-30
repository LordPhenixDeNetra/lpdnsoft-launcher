//! Tests du contrat public : lecture du JSON d'une release.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{AppSpec, AssetPattern, Packages};
use lpdnsoft_launcher_github::{parse_release, parse_releases};

fn app() -> AppSpec {
    AppSpec {
        id: "ankou".into(),
        name: "aNkou".into(),
        tagline: "VPN".into(),
        author: "LPDNSoft".into(),
        repo: "LordPhenixDeNetra/ankou".into(),
        homepage: None,
        asset: AssetPattern::new("ankou-{version}-{os}-{arch}.{ext}"),
        packages: Packages::default(),
        hidden: false,
    }
}

const RELEASE: &str = r#"{
    "tag_name": "v1.0.1",
    "published_at": "2026-10-28T09:12:00Z",
    "html_url": "https://github.com/LordPhenixDeNetra/ankou/releases/tag/v1.0.1",
    "assets": [
        {"name": "ankou-1.0.1-windows-x86_64.msi", "browser_download_url": "https://example.test/a.msi", "size": 4194304},
        {"name": "SHA256SUMS", "browser_download_url": "https://example.test/SHA256SUMS", "size": 231}
    ]
}"#;

#[test]
fn maps_a_github_release() {
    let release = parse_release(RELEASE, &app()).unwrap();
    assert_eq!(release.version.to_string(), "1.0.1");
    assert_eq!(release.published.unwrap().short(), "28/10/2026");
    assert_eq!(release.assets.len(), 2);
    assert_eq!(release.checksums().unwrap().name, "SHA256SUMS");
    assert!(release.notes_url.ends_with("/releases/tag/v1.0.1"));
}

#[test]
fn refuses_a_tag_that_is_not_a_version() {
    assert!(matches!(
        parse_release(r#"{"tag_name": "nightly"}"#, &app()),
        Err(lpdnsoft_launcher_github::GithubError::Version(_))
    ));
}

#[test]
fn rejects_a_body_that_is_not_json() {
    assert!(matches!(
        parse_release("pas du JSON", &app()),
        Err(lpdnsoft_launcher_github::GithubError::Decode(_))
    ));
}

#[test]
fn accepts_a_release_without_assets_or_date() {
    let release = parse_release(r#"{"tag_name": "v0.1.0"}"#, &app()).unwrap();
    assert!(release.assets.is_empty());
    assert!(release.published.is_none());
    assert_eq!(release.notes_url, app().notes_url());
}

#[test]
fn reads_the_notes_of_a_release() {
    let release = parse_release(
        r#"{"tag_name": "v1.0.1", "body": "  ## Corrigé\n\n- la reprise après coupure  "}"#,
        &app(),
    )
    .unwrap();
    assert_eq!(
        release.notes.as_deref(),
        Some("## Corrigé\n\n- la reprise après coupure"),
        "les espaces autour du texte sont retirés, le contenu est gardé tel quel"
    );
}

#[test]
fn an_empty_body_means_no_notes() {
    for body in [r#""body": """#, r#""body": "   ""#, ""] {
        let json = format!(
            r#"{{"tag_name": "v1.0.1"{}{}}}"#,
            if body.is_empty() { "" } else { ", " },
            body
        );
        let release = parse_release(&json, &app()).unwrap();
        assert!(release.notes.is_none(), "corps vide : {json}");
    }
}

#[test]
fn reads_a_list_of_releases_in_order() {
    let json = r#"[
        {"tag_name": "v1.0.1", "body": "nouvelle"},
        {"tag_name": "v1.0.0", "body": "première"}
    ]"#;
    let releases = parse_releases(json, &app()).unwrap();
    assert_eq!(releases.len(), 2);
    assert_eq!(releases[0].version.to_string(), "1.0.1");
    assert_eq!(releases[1].version.to_string(), "1.0.0");
    assert_eq!(releases[1].notes.as_deref(), Some("première"));
}

#[test]
fn skips_an_unreadable_entry_instead_of_losing_the_list() {
    let json = r#"[
        {"tag_name": "pas-une-version"},
        {"tag_name": "v1.0.0"}
    ]"#;
    let releases = parse_releases(json, &app()).unwrap();
    assert_eq!(releases.len(), 1, "l'entrée illisible est ignorée");
    assert_eq!(releases[0].version.to_string(), "1.0.0");
}
