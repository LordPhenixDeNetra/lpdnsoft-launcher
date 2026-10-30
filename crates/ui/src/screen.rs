//! Écran principal : la liste des applications (référence visuelle : launcher ArtCraft).

use crate::fixture;
use egui::{
    Align, Align2, Button, Color32, CornerRadius, FontId, Frame, Grid, Layout, RichText,
    ScrollArea, Sense, Stroke, TextEdit, Ui, vec2,
};
use lpdnsoft_i18n::{Localizer, t};
use lpdnsoft_launcher_core::{Catalog, Platform, Row, State};
use lpdnsoft_ui::{Tone, status_color};

/// Ce que l'écran demande au reste du programme. Rien n'est exécuté en v0.1 : le
/// téléchargement et l'installation viendront derrière cette file (ADR 0001).
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

pub struct Screen {
    pub loc: Localizer,
    pub platform: Platform,
    pub catalog: Catalog,
    pub query: String,
    pub outbox: Vec<Request>,
}

impl Screen {
    pub fn new(loc: Localizer) -> Self {
        Self {
            loc,
            platform: Platform::current(),
            catalog: fixture::catalog(),
            query: String::new(),
            outbox: Vec::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        self.header(ui);
        ui.add_space(12.0);
        self.table(ui);
        self.footer(ui);
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
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
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
                            ui.label(
                                RichText::new(t!(
                                    &self.loc,
                                    "ui-empty",
                                    query = self.query.as_str()
                                ))
                                .weak(),
                            );
                            ui.end_row();
                        }
                        for row in rows {
                            self.row(ui, &row);
                            ui.end_row();
                        }
                    });
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
        ui.horizontal(|ui| {
            if ui.add(Button::new(RichText::new(label).strong())).clicked() {
                self.outbox.push(request.clone());
            }
            ui.menu_button("...", |ui| self.menu_items(ui, row));
        });
    }

    fn menu_items(&mut self, ui: &mut Ui, row: &Row) {
        let id = row.app.id.clone();
        if ui.button(t!(&self.loc, "ui-menu-version")).clicked() {
            self.outbox.push(Request::ChooseVersion(id.clone()));
            ui.close();
        }
        if ui.button(t!(&self.loc, "ui-menu-notes")).clicked() {
            self.outbox.push(Request::Notes(id.clone()));
            ui.close();
        }
        if ui.button(t!(&self.loc, "ui-menu-website")).clicked() {
            self.outbox.push(Request::Website(id.clone()));
            ui.close();
        }
        if ui.button(t!(&self.loc, "ui-menu-repo")).clicked() {
            self.outbox.push(Request::Repository(id.clone()));
            ui.close();
        }
        if ui.button(t!(&self.loc, "ui-menu-all-versions")).clicked() {
            self.outbox.push(Request::AllVersions(id.clone()));
            ui.close();
        }
        if ui.button(t!(&self.loc, "ui-menu-hide")).clicked() {
            self.outbox.push(Request::Hide(id));
            ui.close();
        }
    }

    fn footer(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(t!(&self.loc, "ui-foot-checked"))
                    .size(11.0)
                    .weak(),
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn renders_the_catalog() {
        let mut screen = Screen::new(crate::localizer(Some("fr")).unwrap());
        let harness = Harness::new_ui(move |ui| screen.show(ui));
        harness.get_by_label("aNkou");
    }

    #[test]
    fn counts_pending_updates() {
        let screen = Screen::new(crate::localizer(Some("fr")).unwrap());
        assert_eq!(screen.catalog.update_count(), 2);
        assert_eq!(screen.catalog.rows().len(), 4);
    }
}
