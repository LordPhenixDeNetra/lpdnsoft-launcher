//! Tests du contrat public : lecture d'un `SHA256SUMS`, empreintes, décisions de reprise.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_download::{Checksums, Plan, part_path, plan, sha256_file, sha256_hex};
use std::path::Path;

const SUMS: &str = "\
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  ankou-1.0.1-windows-x86_64.msi
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 *ankou-1.0.1-linux-x86_64.deb
";

#[test]
fn reads_a_checksum_file() {
    let sums = Checksums::parse(SUMS);
    assert_eq!(sums.len(), 2);
    assert_eq!(
        sums.get("ankou-1.0.1-windows-x86_64.msi"),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    );
    // Le `*` du mode binaire ne fait pas partie du nom.
    assert!(sums.get("ankou-1.0.1-linux-x86_64.deb").is_some());
}

#[test]
fn ignores_lines_that_are_not_checksums() {
    let text = "\
# commentaire
ligne sans empreinte
0123  trop-court
not-a-digest  fichier
";
    assert!(Checksums::parse(text).is_empty());
}

#[test]
fn compares_digests_without_caring_about_case() {
    let sums = Checksums::parse(
        "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD  paquet",
    );
    assert_eq!(
        sums.get("paquet"),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    );
}

#[test]
fn the_partial_file_sits_next_to_the_destination() {
    let part = part_path(Path::new("C:/cache/paquet.msi"));
    assert_eq!(part.file_name().unwrap(), "paquet.msi.part");
}

#[test]
fn hashes_known_values() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn hashes_a_file_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paquet");
    std::fs::write(&path, b"abc").unwrap();
    assert_eq!(
        sha256_file(&path).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn decides_how_to_handle_each_response() {
    // Rien de téléchargé : on reprend à zéro.
    assert_eq!(plan(0, 200), Plan::Restart);
    // Un fichier partiel et un 206 : on ajoute la suite.
    assert_eq!(plan(1024, 206), Plan::Append);
    // Un serveur qui ignore `Range` renvoie 200 : on recommence.
    assert_eq!(plan(1024, 200), Plan::Restart);
    // La plage demandée n'existe plus : on recommence aussi.
    assert_eq!(plan(1024, 416), Plan::Restart);
    // Un 206 sans rien à reprendre est incohérent.
    assert_eq!(plan(0, 206), Plan::Refuse(206));
    assert_eq!(plan(0, 404), Plan::Refuse(404));
}
