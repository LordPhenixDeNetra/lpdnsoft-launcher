//! Tests du contrat public : téléchargement, reprise et vérification, contre un serveur
//! HTTP minimal tenu par le test lui-même — aucun réseau externe.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_download::{Download, DownloadError, part_path, sha256_hex};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// Serveur de test : répond une fois par connexion, honore `Range` si demandé.
struct Server {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

fn serve(status: u16, body: &'static [u8], honour_range: bool) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let log = requests.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut raw = Vec::new();
            let mut buffer = [0u8; 2048];
            loop {
                let read = stream.read(&mut buffer).unwrap_or(0);
                if read == 0 {
                    break;
                }
                raw.extend_from_slice(&buffer[..read]);
                if raw.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&raw).to_string();
            log.lock().unwrap().push(request.clone());
            let start = request.lines().find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("range: bytes=")
                    .and_then(|value| value.trim().trim_end_matches('-').parse::<usize>().ok())
            });
            let (code, payload) = match (status, honour_range, start) {
                (200, true, Some(start)) if start < body.len() => (206, &body[start..]),
                _ => (status, body),
            };
            let header = format!(
                "HTTP/1.1 {code} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                payload.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(payload);
            let _ = stream.flush();
        }
    });
    Server {
        url: format!("http://127.0.0.1:{port}/paquet.bin"),
        requests,
    }
}

#[tokio::test]
async fn downloads_a_package_and_publishes_it() {
    let server = serve(200, b"bonjour", false);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paquet.bin");
    let client = reqwest::Client::new();

    let written = Download::new(&client, &server.url, &path, &sha256_hex(b"bonjour"))
        .run(|_| {})
        .await
        .unwrap();

    assert_eq!(written, 7);
    assert_eq!(std::fs::read(&path).unwrap(), b"bonjour");
    assert!(!part_path(&path).exists(), "le fichier partiel est publié");
}

#[tokio::test]
async fn resumes_where_the_previous_attempt_stopped() {
    let server = serve(200, b"bonjour", true);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paquet.bin");
    std::fs::write(part_path(&path), b"bon").unwrap();
    let client = reqwest::Client::new();

    let written = Download::new(&client, &server.url, &path, &sha256_hex(b"bonjour"))
        .run(|_| {})
        .await
        .unwrap();

    assert_eq!(written, 7);
    assert_eq!(std::fs::read(&path).unwrap(), b"bonjour");
    let request = server.requests.lock().unwrap().first().cloned().unwrap();
    assert!(
        request.to_ascii_lowercase().contains("range: bytes=3-"),
        "la reprise doit demander la suite : {request}"
    );
}

#[tokio::test]
async fn refuses_a_package_whose_checksum_differs() {
    let server = serve(200, b"bonjour", false);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paquet.bin");
    let client = reqwest::Client::new();

    let error = Download::new(&client, &server.url, &path, &"0".repeat(64))
        .run(|_| {})
        .await
        .unwrap_err();

    assert!(
        matches!(error, DownloadError::ChecksumMismatch { .. }),
        "{error:?}"
    );
    assert!(!path.exists(), "un paquet non vérifié n'est jamais publié");
    assert!(!part_path(&path).exists(), "le partiel est supprimé");
}

#[tokio::test]
async fn refuses_a_package_of_the_wrong_size() {
    let server = serve(200, b"bonjour", false);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paquet.bin");
    let client = reqwest::Client::new();

    let error = Download::new(&client, &server.url, &path, &sha256_hex(b"bonjour"))
        .with_size(999)
        .run(|_| {})
        .await
        .unwrap_err();

    assert!(
        matches!(error, DownloadError::SizeMismatch { .. }),
        "{error:?}"
    );
    assert!(!path.exists());
}

#[tokio::test]
async fn reports_a_package_that_does_not_exist() {
    let server = serve(404, b"", false);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paquet.bin");
    let client = reqwest::Client::new();

    let error = Download::new(&client, &server.url, &path, &"0".repeat(64))
        .run(|_| {})
        .await
        .unwrap_err();

    match error {
        DownloadError::Network(message) => assert!(message.contains("404"), "{message}"),
        other => panic!("erreur réseau attendue, reçu {other:?}"),
    }
    assert!(!path.exists());
}
