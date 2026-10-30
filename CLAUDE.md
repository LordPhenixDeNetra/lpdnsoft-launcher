# lpdnsoft-launcher — consignes pour les agents

Launcher de la suite **LPDNSoft** (Rust, egui 0.36, Windows/Linux/macOS). Décisions :
`docs/adr/`.

## Disposition des dépôts

`lpdnsoft-launcher` et `lpdnsoft-libs` sont clonés **côte à côte** dans le même dossier parent ;
`lpdnsoft-launcher` dépend de `../lpdnsoft-libs/crates/<nom>`.

## Structure

- `crates/core` — `lpdnsoft-launcher-core` : versions, manifeste `apps.json`, releases, états,
  filtrage. **Aucune E/S, aucun réseau.**
- `crates/ui` — `lpdnsoft-launcher-ui` : fenêtre eframe, liste, pastilles d'état, menu par
  application ; `fixture.rs` fournit les données de démonstration.
- `apps/launcher` — `lpdnsoft-launcher` : arguments (`--lang en|fr`), journal, lancement.
- `locales/{en,fr}/launcher.ftl` — traductions Fluent, embarquées à la compilation.
- `xtask` — contrôles.

## Commande unique

```sh
cargo xtask ci   # i18n complet, fmt --check, clippy -D warnings, tests, cargo deny check
```

Prérequis : Rust stable ≥ 1.95, `cargo install cargo-deny --locked`.

## Règles

- Édition 2024, `resolver = "3"`, licence `MIT OR Apache-2.0`.
- `unsafe_code = "forbid"` ; clippy `unwrap_used`, `expect_used`, `dbg_macro`, `todo` en
  `deny` (autorisés dans les tests).
- Une erreur `thiserror` par crate ; `anyhow` seulement dans `apps/*` et `xtask`.
- Préfixe des crates : `lpdnsoft-`.
- Langues : `en` et `fr`, repli `en`.

## i18n

Aucune chaîne visible en dur : tout passe par `t!(&loc, "id", arg = valeur)`. Ajouter chaque
clé dans `locales/en/launcher.ftl` **et** `locales/fr/launcher.ftl` — une clé absente casse la
compilation.

## État et périmètre

v0.1 est un **socle sans effet de bord** : pas de réseau, pas de téléchargement, pas
d'installation, pas de détection des versions installées. Les boutons remplissent une file de
`Request` que rien ne consomme encore. Ne pas présenter l'inverse dans la documentation.
