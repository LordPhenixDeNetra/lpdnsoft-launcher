//! Assemblage : fenêtre eframe et écran principal.

use crate::UiError;
use crate::localizer;
use crate::screen::Screen;
use lpdnsoft_i18n::t;

struct App {
    screen: Screen,
}

/// Lance la fenêtre ; `lang` (`--lang`) l'emporte sur la langue du système.
pub fn run(lang: Option<String>) -> Result<(), UiError> {
    let loc = localizer(lang.as_deref())?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 720.0])
            .with_min_inner_size([980.0, 480.0])
            .with_title(t!(&loc, "app-title")),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "lpdnsoft-launcher",
        options,
        Box::new(move |cc| {
            lpdnsoft_ui::install(&cc.egui_ctx);
            // La référence visuelle est sombre (ADR 0001) ; le thème commun gère les deux.
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(App {
                screen: Screen::new(loc),
            }))
        }),
    )
    .map_err(|e| UiError::Display(e.to_string()))
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::group(ui.style()).inner_margin(16.0))
            .show(ui, |ui| self.screen.show(ui));
    }
}
