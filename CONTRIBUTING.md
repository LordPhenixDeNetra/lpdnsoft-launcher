# Contribuer au launcher LPDNSoft

Merci de vouloir y contribuer. Ce dépôt contient le launcher de la suite **LPDNSoft** (Rust,
egui, Windows/Linux/macOS). Les décisions d'architecture sont dans `docs/adr/`.

## Prérequis

- Rust stable **≥ 1.95** (édition 2024, `resolver = "3"`) ;
- `cargo install cargo-deny --locked` ;
- le dépôt `lpdnsoft-libs` cloné **à côté** de celui-ci : le launcher dépend de
  `../lpdnsoft-libs/crates/<nom>`.

```
N:\dev\LPDNSoft\lpdnsoft-launcher
N:\dev\LPDNSoft\lpdnsoft-libs
```

## Construire, lancer, tester

```sh
cargo run -p lpdnsoft-launcher          # lance le launcher
cargo xtask ci                          # i18n, fmt, clippy, tests, cargo deny
```

**`cargo xtask ci` est la seule commande à connaître.** Elle vérifie que chaque clé de
traduction existe dans les deux langues, que le formatage est fait, que clippy ne signale
rien (`-D warnings`), que tous les tests passent, et que les licences des dépendances sont
acceptables. Une contribution qui échoue là n'est pas prête.

## Règles de code

- **`unsafe` est interdit** (`unsafe_code = "forbid"`), et clippy refuse `unwrap`, `expect`,
  `dbg!` et `todo!` hors des tests.
- **Aucune chaîne visible en dur** : tout passe par `t!(&loc, "id", …)`, et chaque clé est
  ajoutée dans `locales/en/launcher.ftl` **et** `locales/fr/launcher.ftl`.
- **Une erreur `thiserror` par crate** ; `anyhow` seulement dans `apps/*` et `xtask`.
- **Une décision d'architecture s'écrit** : un fichier dans `docs/adr/`, avec le contexte, les
  options écartées et les conséquences. Le code suit la décision, jamais l'inverse.
- **Le suivi vit dans `BACKLOG.md`** : une tâche commencée y passe en *En cours*, une tâche
  livrée en *Fait* avec son numéro de PR, dans le même changement que le code.

## Tests

Aucun travail n'est livré sans ses tests.

- **Placer le test selon sa nature** : la logique privée (une fonction interne) se teste dans
  un `#[cfg(test)] mod tests` au bas du fichier ; l'API publique se teste dans
  `crates/<crate>/tests/*.rs`. Un test d'intégration ne voit que ce que le crate expose : on
  n'élargit jamais une API pour pouvoir la tester.
- **Un test nouveau doit échouer sans le changement**, sinon il ne prouve rien.
- **Ne jamais désactiver un test existant** pour faire passer une modification : le corriger,
  ou expliquer pourquoi il n'a plus de sens.
- Les tests qui touchent le réseau, un système ou un installeur réel sont lancés à la main, et
  le résultat est consigné dans `docs/runbooks/`.

## Commits

- **Format** : [Conventional Commits](https://www.conventionalcommits.org/), en anglais, avec
  une portée quand elle est utile : `feat(ui):`, `fix(system):`, `docs:`, `test(download):`.
- **Un commit par sujet**, et `git add` avec des chemins explicites (jamais `git add .`).
- **Signer** : `git commit -s` ajoute la ligne `Signed-off-by` exigée par le DCO.
- **Un `-m` par paragraphe** : le premier est le titre, les suivants expliquent le pourquoi,
  pas le comment.

## Branches et demande de fusion

- Nommer la branche `feat/…`, `fix/…`, `docs/…` ou `test/…`.
- Ouvrir la demande de fusion vers `main` **après** un `cargo xtask ci` vert.
- Le titre de la PR suit le format des commits ; la description reprend le formulaire
  `.github/pull_request_template.md`.
- Une PR qui change l'interface joint une capture `docs/capture.png` à jour.

## Signaler un problème

Ouvrez une *issue* avec : ce que vous faisiez, ce que vous attendiez, ce qui s'est passé, et
la version affichée en pied de fenêtre. Les journaux se trouvent dans le dossier de
configuration du launcher.
