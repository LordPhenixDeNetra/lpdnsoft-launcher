//! Écran principal : la liste des applications (référence visuelle : launcher ArtCraft).

use egui::{
    Align, Align2, Button, Color32, CornerRadius, FontId, Frame, Grid, Layout, RichText, Sense,
    Stroke, TextEdit, Ui, vec2,
};
use lpdnsoft_i18n::{Localizer, t};
use lpdnsoft_launcher_core::{Catalog, Platform, Release, Row, State};
use lpdnsoft_ui::{Tone, status_color};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::links;

/// Ce que l'écran demande au reste du programme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Refresh,
    UpdateAll,
    Install(String),
    Update(String),
    Open(String),
    ChooseVersion(String),
    Notes(String),
    Website(String),
    Repository(String),
    AllVersions(String),
    Hide(String),
}

/// Étape d'une installation en cours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Downloading,
    Installing,
    Done,
    Failed(String),
}

/// Ce que le pont raconte pendant une installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub app: String,
    pub step: Step,
    pub done: u64,
    pub total: Option<u64>,
}

/// Une version dans la fenêtre de détail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailEntry {
    pub version: String,
    pub date: Option<String>,
    pub notes: Option<String>,
}

/// Fenêtre de détail : notes de la version publiée, ou historique complet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detail {
    /// Identifiant de l'application, pour rattacher la réponse du pont.
    pub id: String,
    pub title: String,
    pub entries: Vec<DetailEntry>,
    pub loading: bool,
    pub error: Option<String>,
}

/// Ce que le pont renvoie à l'écran.
#[derive(Debug, Clone, PartialEq)]
pub enum Update {
    Checking,
    Catalog {
        rows: Vec<Row>,
        cached: bool,
        /// Application → exécutable réellement présent sur le disque.
        openable: HashMap<String, PathBuf>,
    },
    Job(Job),
    /// Historique chargé pour une application.
    Versions {
        app: String,
        releases: Vec<Release>,
    },
    /// Historique impossible à charger ; la raison est affichée telle quelle.
    VersionsFailed {
        app: String,
        detail: String,
    },
    Failed(String),
}

/// État de la dernière vérification, affiché en pied de page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    Never,
    Running,
    Done,
    /// Vérifiée, mais au moins une réponse vient du cache : à dire, pas à cacher.
    DoneCached,
    Failed(String),
}

pub struct Screen {
    pub loc: Localizer,
    pub platform: Platform,
    pub catalog: Catalog,
    pub query: String,
    pub outbox: Vec<Request>,
    pub check: Check,
    /// Échec d'une action immédiate (navigateur indisponible), effacé à la passe suivante.
    pub notice: Option<String>,
    /// Installation en cours, ou dernière installation terminée.
    pub job: Option<Job>,
    /// Exécutables trouvés lors de la dernière vérification.
    pub openable: HashMap<String, PathBuf>,
    /// Fenêtre de détail ouverte, s'il y en a une.
    pub detail: Option<Detail>,
}

impl Screen {
    pub fn new(loc: Localizer) -> Self {
        Self {
            loc,
            platform: Platform::current(),
            catalog: Catalog::default(),
            query: String::new(),
            outbox: Vec::new(),
            check: Check::Never,
            notice: None,
            job: None,
            openable: HashMap::new(),
            detail: None,
        }
    }

    pub fn apply(&mut self, update: Update) {
        self.notice = None;
        match update {
            Update::Checking => {
                // Une nouvelle vérification efface l'installation terminée.
                self.job = None;
                self.check = Check::Running;
            }
            Update::Catalog {
                rows,
                cached,
                openable,
            } => {
                self.catalog = Catalog::new(rows);
                self.openable = openable;
                self.check = if cached {
                    Check::DoneCached
                } else {
                    Check::Done
                };
            }
            Update::Job(job) => self.job = Some(job),
            Update::Versions { app, releases } => {
                if let Some(detail) = &mut self.detail
                    && detail.id == app
                {
                    detail.loading = false;
                    detail.entries = releases
                        .into_iter()
                        .map(|release| DetailEntry {
                            version: release.version.to_string(),
                            date: release.published.map(|date| date.short()),
                            notes: release.notes,
                        })
                        .collect();
                }
            }
            Update::VersionsFailed {
                app,
                detail: reason,
            } => {
                if let Some(detail) = &mut self.detail
                    && detail.id == app
                {
                    detail.loading = false;
                    detail.error = Some(reason);
                }
            }
            Update::Failed(detail) => self.check = Check::Failed(detail),
        }
    }

    /// Ouvre la fenêtre sur les notes de la version publiée. Aucune requête : le texte est
    /// déjà dans la release, donc dans le cache (ADR 0003).
    pub fn open_notes(&mut self, id: &str) {
        let Some(row) = self.catalog.get(id) else {
            return;
        };
        let Some(release) = row.available.clone() else {
            return;
        };
        self.detail = Some(Detail {
            id: id.to_owned(),
            title: format!("{} — {}", row.app.name, release.version),
            entries: vec![DetailEntry {
                version: release.version.to_string(),
                date: release.published.map(|date| date.short()),
                notes: release.notes.clone(),
            }],
            loading: false,
            error: None,
        });
    }

    /// Ouvre la fenêtre sur l'historique, encore vide ; le pont le remplit.
    pub fn open_history(&mut self, id: &str) {
        let Some(row) = self.catalog.get(id) else {
            return;
        };
        let title = format!("{} — {}", row.app.name, t!(&self.loc, "ui-detail-history"));
        self.detail = Some(Detail {
            id: id.to_owned(),
            title,
            entries: Vec::new(),
            loading: true,
            error: None,
        });
    }

    /// Fenêtre de détail : notes d'une version, ou historique complet.
    fn detail_window(&mut self, ui: &mut Ui) {
        let Some(detail) = self.detail.clone() else {
            return;
        };
        let mut open = true;
        egui::Window::new(&detail.title)
            .open(&mut open)
            .collapsible(false)
            .default_width(560.0)
            .max_height(460.0)
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical()
                    .max_height(360.0)
                    .show(ui, |ui| {
                        if detail.loading {
                            ui.label(t!(&self.loc, "ui-detail-loading"));
                            return;
                        }
                        if let Some(error) = &detail.error {
                            ui.label(
                                RichText::new(t!(
                                    &self.loc,
                                    "ui-detail-failed",
                                    detail = error.as_str()
                                ))
                                .color(status_color(Tone::Alert)),
                            );
                            return;
                        }
                        if detail.entries.is_empty() {
                            ui.label(t!(&self.loc, "ui-detail-empty-history"));
                            return;
                        }
                        for entry in &detail.entries {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&entry.version).strong());
                                if let Some(date) = &entry.date {
                                    ui.label(RichText::new(date).size(11.0).weak());
                                }
                            });
                            match &entry.notes {
                                Some(notes) => {
                                    ui.label(RichText::new(notes).size(12.0));
                                }
                                None => {
                                    ui.label(
                                        RichText::new(t!(&self.loc, "ui-detail-no-notes"))
                                            .size(12.0)
                                            .weak(),
                                    );
                                }
                            }
                            ui.add_space(10.0);
                        }
                    });
            });
        if !open {
            self.detail = None;
        }
    }

    pub fn mark_checking(&mut self) {
        self.check = Check::Running;
    }

    pub fn show(&mut self, ui: &mut Ui) {
        self.header(ui);
        ui.add_space(12.0);
        self.table(ui);
        self.job_line(ui);
        self.detail_window(ui);
        self.footer(ui);
    }

    /// Ligne d'état de l'installation en cours, entre la liste et le pied de page.
    fn job_line(&self, ui: &mut Ui) {
        let Some(job) = &self.job else {
            return;
        };
        let tone = match &job.step {
            Step::Downloading | Step::Installing => Tone::Busy,
            Step::Done => Tone::On,
            Step::Failed(_) => Tone::Alert,
        };
        let text = match &job.step {
            Step::Downloading => {
                let percent = match job.total {
                    Some(total) if total > 0 => (job.done.saturating_mul(100) / total).min(100),
                    _ => 0,
                };
                t!(
                    &self.loc,
                    "ui-job-downloading",
                    app = job.app.as_str(),
                    percent = percent as i64
                )
            }
            Step::Installing => t!(&self.loc, "ui-job-installing", app = job.app.as_str()),
            Step::Done => t!(&self.loc, "ui-job-done", app = job.app.as_str()),
            Step::Failed(detail) => t!(
                &self.loc,
                "ui-job-failed",
                app = job.app.as_str(),
                detail = detail.as_str()
            ),
        };
        ui.add_space(6.0);
        ui.label(RichText::new(text).size(12.0).color(status_color(tone)));
    }

    fn header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(t!(&self.loc, "ui-brand")).size(20.0).strong());
            ui.add_space(10.0);
            ui.label(
                RichText::new(t!(&self.loc, "ui-brand-sub"))
                    .size(11.0)
                    .weak(),
            );
            ui.add_space(12.0);
            if ui.button(t!(&self.loc, "ui-refresh")).clicked() {
                self.outbox.push(Request::Refresh);
            }
            ui.add_space(16.0);
            ui.add_sized(
                vec2(260.0, 26.0),
                TextEdit::singleline(&mut self.query).hint_text(t!(&self.loc, "ui-search")),
            );
            ui.add_space(12.0);
            let updates = self.catalog.update_count();
            let label = t!(&self.loc, "ui-update-all", count = updates as i64);
            let button = Button::new(RichText::new(label).strong());
            if ui.add_enabled(updates > 0, button).clicked() {
                self.outbox.push(Request::UpdateAll);
            }
        });
    }

    fn table(&mut self, ui: &mut Ui) {
        // Pas de zone défilante pour l'instant : avec deux applications elle prenait toute
        // la hauteur et repoussait le pied de page hors de la fenêtre. Le défilement
        // viendra quand la liste dépassera la fenêtre, avec une hauteur bornée explicite.
        Grid::new("launcher-apps")
            .num_columns(6)
            .spacing(vec2(18.0, 10.0))
            .min_col_width(0.0)
            .show(ui, |ui| {
                self.headers(ui);
                ui.end_row();
                let rows: Vec<Row> = self
                    .catalog
                    .visible(&self.query)
                    .into_iter()
                    .cloned()
                    .collect();
                if rows.is_empty() {
                    let text = if matches!(self.check, Check::Running) {
                        t!(&self.loc, "ui-loading")
                    } else {
                        t!(&self.loc, "ui-empty", query = self.query.as_str())
                    };
                    ui.label(RichText::new(text).weak());
                    ui.end_row();
                }
                for row in rows {
                    self.row(ui, &row);
                    ui.end_row();
                }
            });
    }

    fn headers(&self, ui: &mut Ui) {
        let small = |text: String| RichText::new(text).size(11.0).weak();
        ui.label(small(t!(&self.loc, "ui-col-app")));
        ui.label(small(t!(&self.loc, "ui-col-state")));
        ui.label(small(t!(&self.loc, "ui-col-version")));
        ui.label(small(t!(&self.loc, "ui-col-date")));
        ui.label(small(t!(&self.loc, "ui-col-author")));
        ui.label("");
    }

    fn row(&mut self, ui: &mut Ui, row: &Row) {
        let state = row.state();
        let color = status_color(state_tone(state));

        let name_cell = ui.allocate_ui_with_layout(
            vec2(280.0, 44.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                icon(ui, &row.app.name, color);
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&row.app.name).strong().size(15.0));
                    ui.label(RichText::new(&row.app.tagline).small().weak());
                });
            },
        );
        name_cell.response.on_hover_text(&row.app.id);

        let state_label = state_text(&self.loc, state);
        pill(ui, &state_label, color);

        match state {
            State::UpdateAvailable {
                installed,
                available,
            } => {
                ui.label(
                    RichText::new(format!("{installed}  →  {available}"))
                        .monospace()
                        .color(color),
                );
            }
            State::NotInstalled | State::Unknown => {
                let text = row
                    .available
                    .as_ref()
                    .map(|release| release.version.to_string())
                    .unwrap_or_else(|| "—".into());
                ui.label(RichText::new(text).monospace());
            }
            State::UpToDate { version } => {
                ui.label(RichText::new(version.to_string()).monospace());
            }
        }

        let date = row
            .available
            .as_ref()
            .and_then(|release| release.published)
            .map(|date| date.short())
            .unwrap_or_else(|| "—".into());
        ui.label(RichText::new(date).small().weak());
        ui.label(RichText::new(&row.app.author).small());

        let (label, request) = action_of(&self.loc, row, state);
        let openable = self.openable.get(&row.app.id).map(PathBuf::as_path);
        let enabled = links::available(row, &request, openable);
        let hint = t!(&self.loc, "ui-unavailable");
        ui.horizontal(|ui| {
            let response = ui.add_enabled(enabled, Button::new(RichText::new(label).strong()));
            let response = if enabled {
                response
            } else {
                response.on_disabled_hover_text(hint)
            };
            if response.clicked() {
                self.outbox.push(request.clone());
            }
            ui.menu_button("...", |ui| self.menu_items(ui, row));
        });
    }

    fn menu_items(&mut self, ui: &mut Ui, row: &Row) {
        let id = row.app.id.clone();
        let hint = t!(&self.loc, "ui-unavailable");
        let entries = [
            // « Ouvrir » est aussi dans le menu : il reste accessible quand la ligne propose
            // déjà « Mettre à jour », où il n'a pas sa place comme action principale.
            (t!(&self.loc, "ui-menu-open"), Request::Open(id.clone())),
            (
                t!(&self.loc, "ui-menu-version"),
                Request::ChooseVersion(id.clone()),
            ),
            (t!(&self.loc, "ui-menu-notes"), Request::Notes(id.clone())),
            (
                t!(&self.loc, "ui-menu-website"),
                Request::Website(id.clone()),
            ),
            (
                t!(&self.loc, "ui-menu-repo"),
                Request::Repository(id.clone()),
            ),
            (
                t!(&self.loc, "ui-menu-all-versions"),
                Request::AllVersions(id.clone()),
            ),
            (t!(&self.loc, "ui-menu-hide"), Request::Hide(id.clone())),
        ];
        for (label, request) in entries {
            let openable = self.openable.get(&row.app.id).map(PathBuf::as_path);
            if let Some(chosen) = menu_entry(ui, label, request, row, &hint, openable) {
                self.outbox.push(chosen);
                ui.close();
            }
        }
    }

    fn footer(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            let (text, color) = match &self.check {
                Check::Never => (t!(&self.loc, "ui-foot-idle"), None),
                Check::Running => (t!(&self.loc, "ui-foot-checking"), None),
                Check::Done => (t!(&self.loc, "ui-foot-done"), None),
                Check::DoneCached => (
                    t!(&self.loc, "ui-foot-cached"),
                    Some(status_color(Tone::Busy)),
                ),
                Check::Failed(detail) => (
                    t!(&self.loc, "ui-foot-failed", detail = detail.as_str()),
                    Some(status_color(Tone::Alert)),
                ),
            };
            let mut label = RichText::new(text).size(11.0).weak();
            if let Some(color) = color {
                label = label.color(color);
            }
            ui.label(label);
            if let Some(notice) = &self.notice {
                ui.add_space(12.0);
                ui.label(
                    RichText::new(notice)
                        .size(11.0)
                        .color(status_color(Tone::Alert)),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                        .size(11.0)
                        .weak(),
                );
            });
        });
    }
}

fn state_tone(state: State) -> Tone {
    match state {
        State::UpdateAvailable { .. } => Tone::Busy,
        State::NotInstalled => Tone::Off,
        State::UpToDate { .. } => Tone::On,
        State::Unknown => Tone::Alert,
    }
}

fn state_text(loc: &Localizer, state: State) -> String {
    match state {
        State::NotInstalled => t!(loc, "ui-state-not-installed"),
        State::UpdateAvailable { .. } => t!(loc, "ui-state-update-available"),
        State::UpToDate { .. } => t!(loc, "ui-state-up-to-date"),
        State::Unknown => t!(loc, "ui-state-unknown"),
    }
}

fn action_of(loc: &Localizer, row: &Row, state: State) -> (String, Request) {
    let id = row.app.id.clone();
    match state {
        State::NotInstalled => (t!(loc, "ui-install"), Request::Install(id)),
        State::UpdateAvailable { .. } => (t!(loc, "ui-update"), Request::Update(id)),
        State::UpToDate { .. } | State::Unknown => (t!(loc, "ui-open"), Request::Open(id)),
    }
}

/// Entrée de menu : désactivée avec une infobulle quand l'action n'existe pas encore.
fn menu_entry(
    ui: &mut Ui,
    label: String,
    request: Request,
    row: &Row,
    hint: &str,
    openable: Option<&Path>,
) -> Option<Request> {
    let enabled = links::available(row, &request, openable);
    let response = ui.add_enabled(enabled, Button::new(label).frame(false));
    let response = if enabled {
        response
    } else {
        response.on_disabled_hover_text(hint)
    };
    response.clicked().then_some(request)
}

/// Pastille d'état : cadre fin de la couleur de l'état, fond très clair.
fn pill(ui: &mut Ui, text: &str, color: Color32) {
    Frame::group(ui.style())
        .fill(color.gamma_multiply(0.12))
        .stroke(Stroke::new(1.0, color))
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).monospace().color(color));
        });
}

/// Vignette de l'application : carré arrondi avec son initiale.
fn icon(ui: &mut Ui, name: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(9), color);
    let initial = name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        initial,
        FontId::proportional(17.0),
        Color32::WHITE,
    );
}
