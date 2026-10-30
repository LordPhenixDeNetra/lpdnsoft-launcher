# lpdnsoft-launcher — consignes pour les agents

Launcher de la suite **LPDNSoft** (Rust, egui 0.36, Windows/Linux/macOS). Décisions :
`docs/adr/`.

## Disposition des dépôts

`lpdnsoft-launcher` et `lpdnsoft-libs` sont clonés **côte à côte** dans le même dossier parent ;
`lpdnsoft-launcher` dépend de `../lpdnsoft-libs/crates/<nom>`.

## Structure

- `crates/core` — `lpdnsoft-launcher-core` : versions, manifeste `apps.json`, releases, états,
  filtrage. **Aucune E/S, aucun réseau.**
- `crates/github` — `lpdnsoft-launcher-github` : `GET /repos/<propriétaire>/<nom>/releases/latest`,
  une requête par application (ADR 0002).
- `crates/system` — `lpdnsoft-launcher-system` : version installée, lue dans le registre de
  désinstallation sous Windows ; rien hors Windows pour l'instant.
- `crates/ui` — `lpdnsoft-launcher-ui` : fenêtre eframe, liste, pastilles d'état, menu par
  application ; `bridge.rs` fait tourner la vérification sur un fil à part.
- `apps/launcher` — `lpdnsoft-launcher` : arguments (`--lang en|fr`), journal, lancement.
- `apps.json` — la liste des applications de la suite, embarquée et validée au démarrage.
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

## Tests : définition de « terminé »

Aucun travail n'est livré sans ses tests.

- **Ajouter les tests du comportement modifié**, dans le même changement que le code.
- **Les faire passer avant de livrer** : `cargo xtask ci` doit sortir en exit 0 (i18n,
  `fmt --check`, `clippy -D warnings`, tests, `cargo deny check`). Un test rouge ne se
  reporte pas à plus tard.
- **Un test nouveau échoue sans le changement** ; sinon il ne prouve rien.
- **Ne jamais désactiver** un test existant pour faire passer une modification : le
  corriger, ou expliquer pourquoi il n'a plus de sens.
- **Indiquer dans la livraison** la commande lancée et son résultat exact, avec les
  commandes git et le formulaire de PR.

Stratégie et niveaux : `docs/superpowers/specs/2026-10-30-tests-unitaires.md`.
Plan d'application : `docs/superpowers/plans/2026-10-30-plan-tests-unitaires.md`.

## État et périmètre

v0.2 lit de **vraies données** : les releases publiées sur GitHub (`GET /releases/latest`,
une requête par application, soixante par heure sans authentification) et la version
installée, lue dans le registre de désinstallation sous Windows. La liste des applications
est `apps.json`, embarqué et validé au démarrage.

Ce qui n'existe pas encore : le téléchargement, la vérification `SHA256SUMS`, l'installation
et la mise à jour. Les boutons Installer / Mettre à jour / Ouvrir et le menu remplissent une
file de `Request` que rien ne consomme — ne pas présenter l'inverse dans la documentation.
La détection des versions installées n'est pas écrite hors Windows.
