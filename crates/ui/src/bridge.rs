//! Fil entre l'interface et le monde : GitHub et le registre, sur un thread à part.
//!
//! L'interface ne bloque jamais : elle demande une vérification, le fil pousse un `Update`
//! quand il a fini, et `wake` provoque le redessin (même principe que le pont d'aNkou).

use crate::screen::Update;
use lpdnsoft_launcher_core::{AppSpec, Row};
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

/// Interroge GitHub pour chaque application, et le registre pour la version installée.
async fn check(apps: &[AppSpec]) -> Update {
    let client = match lpdnsoft_launcher_github::client() {
        Ok(client) => client,
        Err(e) => return Update::Failed(e.to_string()),
    };
    let mut rows = Vec::with_capacity(apps.len());
    let mut first_error = None;
    for app in apps {
        let installed = lpdnsoft_launcher_system::installed_version(app);
        let available = match lpdnsoft_launcher_github::latest_release(&client, app).await {
            Ok(release) => Some(release),
            Err(e) => {
                first_error.get_or_insert_with(|| e.to_string());
                None
            }
        };
        rows.push(Row::new(app.clone(), installed, available));
    }
    // Tout a échoué : mieux vaut le dire que d'afficher un catalogue vide et muet.
    if rows.iter().all(|row| row.available.is_none()) {
        let detail = first_error.unwrap_or_else(|| "aucune donnée reçue".into());
        return Update::Failed(detail);
    }
    Update::Catalog(rows)
}
