//! Assemblage : fenêtre eframe, pont vers GitHub et le registre.

use crate::UiError;
use crate::bridge::{Bridge, Wake};
use crate::localizer;
use crate::manifest;
use crate::screen::{Request, Screen};
use lpdnsoft_i18n::t;
use std::sync::Arc;

struct App {
    screen: Screen,
    bridge: Bridge,
}

/// Lance la fenêtre ; `lang` (`--lang`) l'emporte sur la langue du système.
pub fn run(lang: Option<String>) -> Result<(), UiError> {
    let loc = localizer(lang.as_deref())?;
    let apps: Vec<_> = manifest()?.visible().cloned().collect();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 720.0])
            .with_min_inner_size([980.0, 480.0])
            .with_title(t!(&loc, "app-title")),
        ..eframe::NativeOptions::default()
    };
    let screen = Screen::new(loc);
    eframe::run_native(
        "lpdnsoft-launcher",
        options,
        Box::new(move |cc| {
            lpdnsoft_ui::install(&cc.egui_ctx);
            // La référence visuelle est sombre (ADR 0001) ; le thème commun gère les deux.
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            let ctx = cc.egui_ctx.clone();
            let wake: Wake = Arc::new(move || ctx.request_repaint());
            let bridge = Bridge::spawn(apps, wake);
            Ok(Box::new(App { screen, bridge }))
        }),
    )
    .map_err(|e| UiError::Display(e.to_string()))
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for update in self.bridge.try_iter() {
            self.screen.apply(update);
        }
        let requests = std::mem::take(&mut self.screen.outbox);
        for request in requests {
            match &request {
                Request::Refresh => {
                    self.screen.mark_checking();
                    self.bridge.refresh();
                }
                Request::Website(id) | Request::Repository(id) => self.open(id.as_str(), &request),
                // Installer, mettre à jour, ouvrir, choisir une version, masquer : pas
                // encore. L'interface les désactive, donc rien n'arrive ici.
                _ => {}
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::group(ui.style()).inner_margin(16.0))
            .show(ui, |ui| self.screen.show(ui));
    }
}

impl App {
    /// Ouvre le site ou le dépôt de l'application ; un échec est affiché, pas avalé.
    fn open(&mut self, id: &str, request: &Request) {
        let Some(row) = self.screen.catalog.get(id) else {
            return;
        };
        let Some(url) = crate::links::url_for(&row.app, request) else {
            return;
        };
        if let Err(e) = crate::open::url(&url) {
            let detail = e.to_string();
            let notice = t!(&self.screen.loc, "ui-open-failed", detail = detail.as_str());
            self.screen.notice = Some(notice);
        }
    }
}
