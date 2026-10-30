//! Tests du contrat public : classification des statuts HTTP.

use lpdnsoft_launcher_github::{GithubError, Outcome, classify};

#[test]
fn accepts_200_as_a_new_release() {
    assert_eq!(classify(200), Ok(Outcome::New));
}

#[test]
fn accepts_304_as_a_still_valid_cache() {
    // Un 304 n'est pas une erreur : c'est la réponse qui ne consomme pas le quota.
    assert_eq!(classify(304), Ok(Outcome::CacheStillValid));
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
