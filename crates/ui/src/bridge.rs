//! Fil entre l'interface et le monde : GitHub et le registre, sur un thread à part.
//!
//! L'interface ne bloque jamais : elle demande une vérification, le fil pousse un `Update`
//! quand il a fini, et `wake` provoque le redessin (même principe que le pont d'aNkou).

use crate::screen::Update;
use lpdnsoft_launcher_core::{AppSpec, Release, Row, Version};
use lpdnsoft_launcher_github::GithubError;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

pub type Wake = Arc<dyn Fn() + Send + Sync>;

pub struct Bridge {
    refresh: Sender<()>,
    updates: Receiver<Update>,
}

impl Bridge {
    pub fn spawn(apps: Vec<AppSpec>, wake: Wake) -> Self {
        let (refresh, requests) = channel::<()>();
        let (tx, updates) = channel();
        std::thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(e) => {
                    let _ = tx.send(Update::Failed(e.to_string()));
                    (wake)();
                    return;
                }
            };
            loop {
                let _ = tx.send(Update::Checking);
                (wake)();
                let update = runtime.block_on(check(&apps));
                let _ = tx.send(update);
                (wake)();
                // Attente de la prochaine demande ; le canal fermé signifie « quitter ».
                if requests.recv().is_err() {
                    return;
                }
            }
        });
        Self { refresh, updates }
    }

    /// Demande une nouvelle vérification (bouton « Actualiser »).
    pub fn refresh(&self) {
        let _ = self.refresh.send(());
    }

    pub fn try_iter(&self) -> impl Iterator<Item = Update> + '_ {
        self.updates.try_iter()
    }
}

/// Ce que les sources ont répondu pour une application.
#[derive(Debug)]
struct Entry {
    app: AppSpec,
    installed: Option<Version>,
    latest: Result<Release, GithubError>,
}

/// Décide ce que l'écran reçoit. Pure : aucune attente réseau ici, donc testable.
fn assemble(entries: Vec<Entry>) -> Update {
    let mut rows = Vec::with_capacity(entries.len());
    let mut first_error = None;
    for entry in entries {
        let available = match entry.latest {
            Ok(release) => Some(release),
            Err(e) => {
                first_error.get_or_insert_with(|| e.to_string());
                None
            }
        };
        rows.push(Row::new(entry.app, entry.installed, available));
    }
    // Tout a échoué : mieux vaut le dire que d'afficher un catalogue vide et muet.
    if rows.iter().all(|row| row.available.is_none()) {
        let detail = first_error.unwrap_or_else(|| "aucune donnée reçue".into());
        return Update::Failed(detail);
    }
    Update::Catalog(rows)
}

/// Interroge GitHub pour chaque application, et le registre pour la version installée.
async fn check(apps: &[AppSpec]) -> Update {
    let client = match lpdnsoft_launcher_github::client() {
        Ok(client) => client,
        Err(e) => return Update::Failed(e.to_string()),
    };
    let mut entries = Vec::with_capacity(apps.len());
    for app in apps {
        entries.push(Entry {
            app: app.clone(),
            installed: lpdnsoft_launcher_system::installed_version(app),
            latest: lpdnsoft_launcher_github::latest_release(&client, app).await,
        });
    }
    assemble(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lpdnsoft_launcher_core::{AssetPattern, Date};

    fn app(id: &str) -> AppSpec {
        AppSpec {
            id: id.into(),
            name: id.to_uppercase(),
            tagline: "Application de test".into(),
            author: "LPDNSoft".into(),
            repo: format!("LordPhenixDeNetra/{id}"),
            homepage: None,
            asset: AssetPattern::new(format!("{id}-{{version}}-{{os}}-{{arch}}.{{ext}}")),
            hidden: false,
        }
    }

    fn release(tag: &str) -> Result<Release, GithubError> {
        Release::new(
            tag,
            Some(Date {
                year: 2026,
                month: 10,
                day: 28,
            }),
            Vec::new(),
            "https://example.test",
        )
        .map_err(GithubError::Version)
    }

    fn entry(id: &str, installed: Option<&str>, latest: Result<Release, GithubError>) -> Entry {
        Entry {
            app: app(id),
            installed: installed.map(|v| v.parse().unwrap()),
            latest,
        }
    }

    #[test]
    fn keeps_the_manifest_order() {
        let update = assemble(vec![
            entry("a", None, release("v1.0.0")),
            entry("b", None, release("v1.0.0")),
            entry("c", None, release("v1.0.0")),
        ]);
        match update {
            Update::Catalog(rows) => {
                let ids: Vec<&str> = rows.iter().map(|row| row.app.id.as_str()).collect();
                assert_eq!(ids, ["a", "b", "c"]);
            }
            other => panic!("catalogue attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn reports_a_failure_only_when_every_source_failed() {
        let update = assemble(vec![
            entry("a", None, Err(GithubError::NoRelease)),
            entry("b", None, Err(GithubError::RateLimited)),
        ]);
        match update {
            Update::Failed(detail) => assert!(detail.contains("release"), "{detail}"),
            other => panic!("échec attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn keeps_a_catalog_when_at_least_one_release_arrived() {
        let update = assemble(vec![
            entry("a", None, release("v1.0.0")),
            entry("b", None, Err(GithubError::RateLimited)),
        ]);
        match update {
            Update::Catalog(rows) => {
                assert_eq!(rows.len(), 2);
                assert!(rows[0].available.is_some());
                assert!(rows[1].available.is_none());
            }
            other => panic!("catalogue attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn carries_the_installed_version_through() {
        let update = assemble(vec![entry("a", Some("1.0.0"), release("v1.0.1"))]);
        match update {
            Update::Catalog(rows) => {
                assert_eq!(rows[0].installed, Some(Version::new(1, 0, 0)));
                assert!(rows[0].is_update());
            }
            other => panic!("catalogue attendu, reçu {other:?}"),
        }
    }
}
