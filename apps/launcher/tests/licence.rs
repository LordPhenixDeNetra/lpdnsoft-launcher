//! Garde-fou : chaque licence déclarée par le manifeste a son texte à la racine.
//!
//! Le 30 octobre 2026, `LICENSE-APACHE` a été supprimé alors que le manifeste annonçait
//! encore `MIT OR Apache-2.0` : un paquet promettait deux licences et n'en livrait qu'une.
//! Ce test existe pour que l'écart ne puisse pas se reproduire.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

/// Déclaration du paquet : `license.workspace = true` la reprend du manifeste racine.
const DECLARED: &str = env!("CARGO_PKG_LICENSE");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn ships_the_text_of_every_declared_licence() {
    assert!(
        root().join("LICENSE-MIT").is_file(),
        "LICENSE-MIT manquant à la racine"
    );
    if DECLARED.contains("Apache-2.0") {
        assert!(
            root().join("LICENSE-APACHE").is_file(),
            "le manifeste annonce Apache-2.0 mais LICENSE-APACHE est absent"
        );
    }
}

#[test]
fn declares_at_least_one_known_licence() {
    assert!(
        DECLARED.contains("MIT") || DECLARED.contains("Apache-2.0"),
        "déclaration inattendue : {DECLARED}"
    );
}
