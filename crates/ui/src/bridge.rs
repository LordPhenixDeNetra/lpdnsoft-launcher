//! Fil entre l'interface et le monde : GitHub, le registre et l'installation, sur un thread
//! à part.
//!
//! L'interface ne bloque jamais : elle demande une vérification ou une installation, le fil
//! pousse un `Update` à chaque étape, et `wake` provoque le redessin.
//!
//! Chaque application est lue en conditionnel : l'empreinte du cache part dans
//! `If-None-Match`, un 304 ne coûte rien au quota, et une panne réseau retombe sur la
//! dernière réponse connue plutôt que sur une liste vide (ADR 0003).

use crate::prepare::prepare;
use crate::screen::{Job, Request, Step, Update};
use lpdnsoft_launcher_cache::{Cache, Entry};
use lpdnsoft_launcher_core::{AppSpec, Platform, Release, Row, Version};
use lpdnsoft_launcher_download::{Checksums, Download};
use lpdnsoft_launcher_github::{Answer, GithubError};
use lpdnsoft_launcher_install::{Executor, SystemExecutor, elevated, plan};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

pub type Wake = Arc<dyn Fn() + Send + Sync>;

pub struct Bridge {
    requests: Sender<Request>,
    updates: Receiver<Update>,
}

impl Bridge {
    pub fn spawn(apps: Vec<AppSpec>, cache: Cache, wake: Wake) -> Self {
        let (requests, inbox) = channel::<Request>();
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
            let mut rows: Vec<Row> = Vec::new();
            loop {
                let _ = tx.send(Update::Checking);
                (wake)();
                let update = runtime.block_on(check(&apps, &cache));
                if let Update::Catalog { rows: current, .. } = &update {
                    rows = current.clone();
                }
                let _ = tx.send(update);
                (wake)();

                // Attente de la prochaine demande ; le canal fermé signifie « quitter ».
                let Ok(request) = inbox.recv() else {
                    return;
                };
                match &request {
                    Request::Install(id) | Request::Update(id) => {
                        if let Some(row) = rows.iter().find(|row| row.app.id == *id).cloned() {
                            runtime.block_on(install(&row, &cache, &tx, &wake));
                        }
                    }
                    // Une requête, seulement à l'ouverture de la fenêtre d'historique.
                    Request::AllVersions(id) => {
                        if let Some(app) = apps.iter().find(|app| app.id == *id).cloned() {
                            runtime.block_on(history(&app, &tx, &wake));
                        }
                    }
                    _ => {}
                }
                // La passe suivante montrera la version nouvellement installée.
            }
        });
        Self { requests, updates }
    }

    /// Transmet une demande de l'écran (vérification, installation).
    pub fn send(&self, request: Request) {
        let _ = self.requests.send(request);
    }

    pub fn try_iter(&self) -> impl Iterator<Item = Update> + '_ {
        self.updates.try_iter()
    }
}

/// Ce que les sources ont répondu pour une application.
#[derive(Debug)]
struct Reading {
    app: AppSpec,
    installed: Option<Version>,
    latest: Result<Release, GithubError>,
    /// Vrai quand la release vient du cache (304 ou panne réseau), pas d'une réponse neuve.
    from_cache: bool,
}

/// Décide ce que l'écran reçoit. Pure : aucune attente réseau ici, donc testable.
fn assemble(readings: Vec<Reading>) -> Update {
    let cached = readings.iter().any(|reading| reading.from_cache);
    let mut rows = Vec::with_capacity(readings.len());
    let mut first_error = None;
    for reading in readings {
        let available = match reading.latest {
            Ok(release) => Some(release),
            Err(e) => {
                first_error.get_or_insert_with(|| e.to_string());
                None
            }
        };
        rows.push(Row::new(reading.app, reading.installed, available));
    }
    // Tout a échoué : mieux vaut le dire qu'afficher un catalogue vide et muet.
    if rows.iter().all(|row| row.available.is_none()) {
        let detail = first_error.unwrap_or_else(|| "aucune donnée reçue".into());
        return Update::Failed(detail);
    }
    Update::Catalog {
        rows,
        cached,
        openable: HashMap::new(),
    }
}

/// Interroge GitHub pour chaque application, et le registre pour la version installée.
async fn check(apps: &[AppSpec], cache: &Cache) -> Update {
    let client = match lpdnsoft_launcher_github::client() {
        Ok(client) => client,
        Err(e) => return Update::Failed(e.to_string()),
    };
    let mut readings = Vec::with_capacity(apps.len());
    for app in apps {
        let installed = lpdnsoft_launcher_system::installed_version(app);
        let known = cache.load(&app.id);
        let etag = known.as_ref().and_then(|entry| entry.etag.clone());
        let answer = lpdnsoft_launcher_github::latest_release(&client, app, etag.as_deref()).await;
        let (latest, from_cache) = match answer {
            Ok(Answer {
                release: Some(release),
                etag,
            }) => {
                let _ = cache.store(
                    &app.id,
                    &Entry {
                        etag,
                        release: release.clone(),
                    },
                );
                (Ok(release), false)
            }
            // 304 : la version en cache reste valable, et la requête ne coûte rien.
            Ok(Answer {
                release: None,
                etag,
            }) => {
                if let (Some(mut entry), Some(etag)) = (known.clone(), etag) {
                    entry.etag = Some(etag);
                    let _ = cache.store(&app.id, &entry);
                }
                match known {
                    Some(entry) => (Ok(entry.release), true),
                    None => (Err(GithubError::NoRelease), false),
                }
            }
            // Panne réseau ou quota : la dernière réponse connue vaut mieux qu'une ligne vide.
            Err(e) => match known {
                Some(entry) => (Ok(entry.release), true),
                None => (Err(e), false),
            },
        };
        readings.push(Reading {
            app: app.clone(),
            installed,
            latest,
            from_cache,
        });
    }
    let mut update = assemble(readings);
    // Les chemins d'exécution sont une lecture du disque : ils sont ajoutés ici, hors de la
    // partie pure qui décide du contenu de la liste.
    if let Update::Catalog { openable, .. } = &mut update {
        for app in apps {
            if let Some(path) = lpdnsoft_launcher_system::installed_path(app) {
                openable.insert(app.id.clone(), path);
            }
        }
    }
    update
}

/// Charge l'historique d'une application, à la demande.
async fn history(app: &AppSpec, tx: &Sender<Update>, wake: &Wake) {
    let update = match lpdnsoft_launcher_github::client() {
        Ok(client) => match lpdnsoft_launcher_github::releases(&client, app).await {
            Ok(releases) => Update::Versions {
                app: app.id.clone(),
                releases,
            },
            Err(error) => Update::VersionsFailed {
                app: app.id.clone(),
                detail: error.to_string(),
            },
        },
        Err(error) => Update::VersionsFailed {
            app: app.id.clone(),
            detail: error.to_string(),
        },
    };
    let _ = tx.send(update);
    (wake)();
}

/// Télécharge, vérifie et installe la ligne ; chaque étape est poussée à l'écran.
async fn install(row: &Row, cache: &Cache, tx: &Sender<Update>, wake: &Wake) {
    let app = row.app.name.clone();
    let step = match install_steps(row, cache, tx, &app, wake).await {
        Ok(()) => Step::Done,
        Err(detail) => Step::Failed(detail),
    };
    let _ = tx.send(Update::Job(Job {
        app,
        step,
        done: 0,
        total: None,
    }));
    (wake)();
}

async fn install_steps(
    row: &Row,
    cache: &Cache,
    tx: &Sender<Update>,
    app: &str,
    wake: &Wake,
) -> Result<(), String> {
    let prepared = prepare(row, Platform::current()).map_err(|e| e.to_string())?;
    let directory = cache
        .directory()
        .and_then(|directory| directory.parent().map(|parent| parent.join("packages")))
        .ok_or_else(|| "dossier de configuration introuvable".to_owned())?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let client = lpdnsoft_launcher_github::client().map_err(|e| e.to_string())?;

    // 1. Les empreintes de la release, puis celle de ce paquet précisément.
    let sums = client
        .get(&prepared.checksums_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let checksum = Checksums::parse(&sums)
        .get(&prepared.name)
        .ok_or_else(|| format!("{} absent de SHA256SUMS", prepared.name))?
        .to_owned();

    // 2. Le paquet, avec reprise et vérification.
    let path = directory.join(&prepared.name);
    let total = prepared.size;
    let _ = tx.send(Update::Job(Job {
        app: app.to_owned(),
        step: Step::Downloading,
        done: 0,
        total,
    }));
    (wake)();
    let mut download = Download::new(&client, &prepared.package_url, &path, &checksum);
    if let Some(size) = total {
        download = download.with_size(size);
    }
    download
        .run(|done| {
            let _ = tx.send(Update::Job(Job {
                app: app.to_owned(),
                step: Step::Downloading,
                done,
                total,
            }));
            (wake)();
        })
        .await
        .map_err(|e| e.to_string())?;

    // 3. Le plan d'installation, puis l'exécution avec l'élévation du système.
    let os = Platform::current().os;
    let base = plan(&path, os).map_err(|e| e.to_string())?;
    let full = elevated(base, os);
    let _ = tx.send(Update::Job(Job {
        app: app.to_owned(),
        step: Step::Installing,
        done: 0,
        total: None,
    }));
    (wake)();
    SystemExecutor.run(&full).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lpdnsoft_launcher_core::{AssetPattern, Date, Packages};

    fn app(id: &str) -> AppSpec {
        AppSpec {
            id: id.into(),
            name: id.to_uppercase(),
            tagline: "Application de test".into(),
            author: "LPDNSoft".into(),
            repo: format!("LordPhenixDeNetra/{id}"),
            homepage: None,
            asset: AssetPattern::new(format!("{id}-{{version}}-{{os}}-{{arch}}.{{ext}}")),
            packages: Packages::default(),
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

    fn reading(
        id: &str,
        installed: Option<&str>,
        latest: Result<Release, GithubError>,
        from_cache: bool,
    ) -> Reading {
        Reading {
            app: app(id),
            installed: installed.map(|v| v.parse().unwrap()),
            latest,
            from_cache,
        }
    }

    #[test]
    fn keeps_the_manifest_order() {
        let update = assemble(vec![
            reading("a", None, release("v1.0.0"), false),
            reading("b", None, release("v1.0.0"), false),
            reading("c", None, release("v1.0.0"), false),
        ]);
        match update {
            Update::Catalog { rows, .. } => {
                let ids: Vec<&str> = rows.iter().map(|row| row.app.id.as_str()).collect();
                assert_eq!(ids, ["a", "b", "c"]);
            }
            other => panic!("catalogue attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn reports_a_failure_only_when_every_source_failed() {
        let update = assemble(vec![
            reading("a", None, Err(GithubError::NoRelease), false),
            reading("b", None, Err(GithubError::RateLimited), false),
        ]);
        match update {
            Update::Failed(detail) => assert!(detail.contains("release"), "{detail}"),
            other => panic!("échec attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn keeps_a_catalog_when_at_least_one_release_arrived() {
        let update = assemble(vec![
            reading("a", None, release("v1.0.0"), false),
            reading("b", None, Err(GithubError::RateLimited), false),
        ]);
        match update {
            Update::Catalog { rows, .. } => {
                assert_eq!(rows.len(), 2);
                assert!(rows[0].available.is_some());
                assert!(rows[1].available.is_none());
            }
            other => panic!("catalogue attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn carries_the_installed_version_through() {
        let update = assemble(vec![reading("a", Some("1.0.0"), release("v1.0.1"), false)]);
        match update {
            Update::Catalog { rows, .. } => {
                assert_eq!(rows[0].installed, Some(Version::new(1, 0, 0)));
                assert!(rows[0].is_update());
            }
            other => panic!("catalogue attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn flags_a_catalog_built_from_cache() {
        let update = assemble(vec![
            reading("a", None, release("v1.0.0"), false),
            reading("b", None, release("v1.0.0"), true),
        ]);
        match update {
            Update::Catalog { cached, .. } => assert!(cached),
            other => panic!("catalogue attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn a_fresh_catalog_is_not_flagged_as_cached() {
        let update = assemble(vec![reading("a", None, release("v1.0.0"), false)]);
        match update {
            Update::Catalog { cached, .. } => assert!(!cached),
            other => panic!("catalogue attendu, reçu {other:?}"),
        }
    }
}
