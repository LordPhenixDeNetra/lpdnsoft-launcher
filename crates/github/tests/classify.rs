//! Tests du contrat public : classification des statuts HTTP.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_github::{GithubError, classify};

#[test]
fn accepts_200() {
    assert_eq!(classify(200), Ok(()));
}

#[test]
fn maps_404_to_no_release() {
    assert_eq!(classify(404), Err(GithubError::NoRelease));
}

#[test]
fn maps_403_and_429_to_rate_limited() {
    assert_eq!(classify(403), Err(GithubError::RateLimited));
    assert_eq!(classify(429), Err(GithubError::RateLimited));
}

#[test]
fn reports_any_other_status_with_its_code() {
    match classify(500) {
        Err(GithubError::Network(message)) => assert!(message.contains("500"), "{message}"),
        other => panic!("réseau attendu, reçu {other:?}"),
    }
}

#[test]
fn does_not_treat_another_success_code_as_a_release() {
    assert!(matches!(classify(204), Err(GithubError::Network(_))));
}
