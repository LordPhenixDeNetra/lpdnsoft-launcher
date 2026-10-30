//! Téléchargement d'un paquet avec vérification d'empreinte (ADR 0004).
//!
//! Trois garanties, dans cet ordre :
//!
//! 1. **Reprise** : le fichier arrive dans `<nom>.part` ; si une tentative précédente l'a
//!    laissé incomplet, la requête suivante demande la suite par `Range` et le serveur
//!    répond 206. Un serveur qui ignore `Range` renvoie 200 : on repart de zéro.
//! 2. **Vérification** : l'empreinte SHA-256 du fichier complet doit être celle publiée dans
//!    `SHA256SUMS`, sinon le fichier est supprimé — jamais laissé à moitié bon.
//! 3. **Publication** : le `.part` n'est renommé qu'une fois taille et empreinte confirmées.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Longueur d'une empreinte SHA-256 en hexadécimal.
const HEX_LEN: usize = 64;

/// En-tête qui demande la suite d'un fichier déjà commencé.
const RANGE: &str = "Range";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DownloadError {
    #[error("réseau : {0}")]
    Network(String),
    #[error("écriture impossible : {0}")]
    Io(String),
    #[error("empreinte absente pour {0}")]
    MissingChecksum(String),
    #[error("empreinte différente pour {name} : attendu {expected}, obtenu {actual}")]
    ChecksumMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    #[error("taille différente pour {name} : attendu {expected} octets, obtenu {actual}")]
    SizeMismatch {
        name: String,
        expected: u64,
        actual: u64,
    },
}

/// Contenu d'un fichier `SHA256SUMS` : nom de fichier → empreinte.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Checksums {
    digests: BTreeMap<String, String>,
}

impl Checksums {
    /// Lit un `SHA256SUMS`. Les lignes illisibles (commentaires, entêtes) sont ignorées :
    /// c'est le nom cherché qui décide, pas la propreté du fichier.
    pub fn parse(text: &str) -> Self {
        let mut digests = BTreeMap::new();
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let (Some(digest), Some(name)) = (parts.next(), parts.next()) else {
                continue;
            };
            if digest.len() != HEX_LEN || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
                continue;
            }
            // Certains outils préfixent le nom d'un `*` pour le mode binaire.
            let name = name.strip_prefix('*').unwrap_or(name);
            digests.insert(name.to_owned(), digest.to_ascii_lowercase());
        }
        Self { digests }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.digests.get(name).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.digests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.digests.is_empty()
    }
}

/// Empreinte SHA-256 d'un contenu, en hexadécimal minuscule.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex(&hasher.finalize())
}

/// Empreinte SHA-256 d'un fichier, lue par blocs : un paquet de 100 Mo ne tient pas en
/// mémoire pour être vérifié.
pub fn sha256_file(path: &Path) -> Result<String, DownloadError> {
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(io)?;
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Ce qu'il faut faire d'une réponse, selon ce qui restait à télécharger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Reprendre à la fin du fichier partiel (206).
    Append,
    /// Repartir de zéro (200, ou 416 quand la plage demandée n'existe plus).
    Restart,
    /// Réponse inutilisable.
    Refuse(u16),
}

/// Décide quoi faire d'une réponse ; pure, donc testable sans serveur.
pub fn plan(already_have: u64, status: u16) -> Plan {
    match status {
        206 if already_have > 0 => Plan::Append,
        200 => Plan::Restart,
        416 => Plan::Restart,
        code => Plan::Refuse(code),
    }
}

/// Chemin du fichier partiel d'une destination.
pub fn part_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}

/// Téléchargement d'un paquet, avec reprise et vérification.
pub struct Download<'a> {
    client: &'a reqwest::Client,
    url: &'a str,
    path: &'a Path,
    checksum: &'a str,
    size: Option<u64>,
}

impl<'a> Download<'a> {
    pub fn new(
        client: &'a reqwest::Client,
        url: &'a str,
        path: &'a Path,
        checksum: &'a str,
    ) -> Self {
        Self {
            client,
            url,
            path,
            checksum,
            size: None,
        }
    }

    /// Taille annoncée du fichier, vérifiée elle aussi quand elle est connue.
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Télécharge, reprend si possible, vérifie, puis publie le fichier.
    ///
    /// `progress` reçoit le nombre d'octets déjà écrits, pour une barre de progression.
    pub async fn run(&self, mut progress: impl FnMut(u64)) -> Result<u64, DownloadError> {
        let part = part_path(self.path);
        let have = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        let mut request = self.client.get(self.url);
        if have > 0 {
            request = request.header(RANGE, format!("bytes={have}-"));
        }
        let response = request
            .send()
            .await
            .map_err(|e| DownloadError::Network(e.to_string()))?;
        let mut written = have;
        match plan(have, response.status().as_u16()) {
            Plan::Append => {}
            Plan::Restart => {
                std::fs::write(&part, b"").map_err(io)?;
                written = 0;
            }
            Plan::Refuse(code) => {
                return Err(DownloadError::Network(format!(
                    "le serveur a répondu {code}"
                )));
            }
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&part)
            .map_err(io)?;
        let mut response = response;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| DownloadError::Network(e.to_string()))?
        {
            file.write_all(&chunk).map_err(io)?;
            written += chunk.len() as u64;
            progress(written);
        }
        file.flush().map_err(io)?;
        drop(file);

        if let Some(expected) = self.size
            && written != expected
        {
            let _ = std::fs::remove_file(&part);
            return Err(DownloadError::SizeMismatch {
                name: name_of(self.path),
                expected,
                actual: written,
            });
        }
        let actual = sha256_file(&part)?;
        if actual != self.checksum.to_ascii_lowercase() {
            let _ = std::fs::remove_file(&part);
            return Err(DownloadError::ChecksumMismatch {
                name: name_of(self.path),
                expected: self.checksum.to_owned(),
                actual,
            });
        }
        std::fs::rename(&part, self.path).map_err(io)?;
        Ok(written)
    }
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn io(e: std::io::Error) -> DownloadError {
    DownloadError::Io(e.to_string())
}
