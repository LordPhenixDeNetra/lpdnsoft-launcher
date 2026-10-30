//! Données de démonstration, en attendant la lecture réelle de GitHub (ADR 0001).
//!
//! Elles reprennent la forme exacte du catalogue réel : le jour où l'API GitHub et la
//! détection des versions installées arrivent, seul ce fichier disparaît.

use lpdnsoft_launcher_core::{AppSpec, AssetPattern, Catalog, Date, Release, Row, Version};

fn app(id: &str, name: &str, tagline: &str, repo: &str) -> AppSpec {
    AppSpec {
        id: id.into(),
        name: name.into(),
        tagline: tagline.into(),
        author: "LPDNSoft".into(),
        repo: repo.into(),
        homepage: Some("https://github.com/LordPhenixDeNetra".into()),
        asset: AssetPattern::new(format!("{id}-{{version}}-{{os}}-{{arch}}.{{ext}}")),
        hidden: false,
    }
}

fn release(app: &AppSpec, tag: &str) -> Release {
    let published = Date {
        year: 2026,
        month: 10,
        day: 28,
    };
    Release::for_app(app, tag, Some(published), Vec::new())
        .unwrap_or_else(|_| unreachable!("étiquette de démonstration"))
}

/// Catalogue affiché au lancement.
pub fn catalog() -> Catalog {
    let ankou = app(
        "ankou",
        "aNkou",
        "Client VPN WireGuard",
        "LordPhenixDeNetra/ankou",
    );
    let alpha = app(
        "demo-alpha",
        "Demo Alpha",
        "Application de démonstration",
        "LordPhenixDeNetra/demo-alpha",
    );
    let beta = app(
        "demo-beta",
        "Demo Beta",
        "Seconde application de démonstration",
        "LordPhenixDeNetra/demo-beta",
    );
    let gamma = app(
        "demo-gamma",
        "Demo Gamma",
        "Troisième application de démonstration",
        "LordPhenixDeNetra/demo-gamma",
    );

    Catalog::new(vec![
        Row::new(
            ankou.clone(),
            Some(Version::new(1, 0, 1)),
            Some(release(&ankou, "v1.0.1")),
        ),
        Row::new(alpha.clone(), None, Some(release(&alpha, "v0.3.0"))),
        Row::new(
            beta.clone(),
            Some(Version::new(0, 8, 0)),
            Some(release(&beta, "v0.14.0")),
        ),
        Row::new(
            gamma.clone(),
            Some(Version::new(0, 7, 0)),
            Some(release(&gamma, "v0.12.0")),
        ),
    ])
}
