// Application fenêtrée sous Windows : pas de console ouverte à côté.
#![cfg_attr(windows, windows_subsystem = "windows")]
//! `lpdnsoft-launcher` : la fenêtre du launcher de la suite LPDNSoft.

use clap::Parser;

#[derive(Parser)]
#[command(name = "lpdnsoft-launcher", about = "LPDNSoft suite launcher")]
struct Cli {
    /// Display language (en, fr); defaults to the system language
    #[arg(long)]
    lang: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let _log = lpdnsoft_log::init(lpdnsoft_log::LogConfig {
        app_name: "lpdnsoft-launcher",
        dir: None,
        default_filter: "warn",
    })?;
    let cli = Cli::parse();
    lpdnsoft_launcher_ui::run(cli.lang)?;
    Ok(())
}
