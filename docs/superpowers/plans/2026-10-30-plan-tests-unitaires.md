# Plan — tests unitaires du launcher

- **Date :** 2026-10-30
- **Spécification :** `docs/superpowers/specs/2026-10-30-tests-unitaires.md`
- **Point de départ :** 26 tests, `cargo xtask ci` vert.
- **Règle :** chaque étape se termine par `cargo xtask ci` en exit 0 ; une étape non verte
  ne passe pas à la suivante.

## État d'avancement

| Étape | État |
|---|---|
| 1 — classification des statuts HTTP | **fait** : `classify`, 5 tests |
| 2 — assemblage du pont | **fait** : `assemble`, 4 tests |
| 3 — aides pures de l'écran | à faire |
| 4 — états de l'écran | à faire |
| 5 — manifeste livré | à faire |
| 6 — vérification manuelle | à faire |
| 7 — verrous CI (`--locked`, MSRV) | **fait** : `xtask`, job `msrv` |
| 8 — artefacts de build | **fait** : job `build` |
| 9 — décision de release | à faire, avec la première fonction d'installation |

**35 tests** au total après les étapes 1 et 2, `cargo xtask ci` vert.

---

## Étape 1 — Rendre la classification des statuts HTTP testable

**Problème.** `latest_release` décide du sort d'une réponse (200, 404, 403, autre) au milieu
d'une fonction asynchrone, donc hors de portée d'un test.

**Changement.** Extraire une fonction pure `classify(status: u16) -> Result<(), GithubError>`
et l'appeler depuis `latest_release`. Aucun changement de comportement.

**Fichiers.** `crates/github/src/lib.rs`

**Tests à ajouter** (dans le crate, sans réseau) :

| Test | Assertion |
|---|---|
| `accepts_200` | `Ok(())` |
| `maps_404_to_no_release` | `Err(NoRelease)` |
| `maps_403_and_429_to_rate_limited` | `Err(RateLimited)` pour les deux |
| `reports_any_other_status_with_its_code` | le code apparaît dans le message |

**Fini quand.** 5 tests de plus, les 26 précédents inchangés, `cargo xtask ci` vert.

---

## Étape 2 — Extraire l'assemblage du pont et le tester

**Problème.** `bridge::check` mêle trois choses : appeler les sources, choisir entre
`Catalog` et `Failed`, construire les lignes. Seule la troisième est testable aujourd'hui.

**Changement.** Séparer :

- `collect()` — asynchrone, appelle GitHub et le registre, rend `Vec<Entry>` ;
- `assemble(entries) -> Update` — **pure**, décide du résultat.

**Fichiers.** `crates/ui/src/bridge.rs`

**Tests à ajouter** :

| Test | Assertion |
|---|---|
| `keeps_the_manifest_order` | trois applications, trois lignes dans l'ordre |
| `reports_a_failure_only_when_every_source_failed` | tout en erreur → `Failed` avec le premier motif |
| `keeps_a_catalog_when_at_least_one_release_arrived` | une réussite, deux échecs → `Catalog` |
| `carries_the_installed_version_through` | version installée conservée à l'identique |

**Fini quand.** `assemble` testée sans async ni réseau, `cargo xtask ci` vert.

---

## Étape 3 — Tester les aides pures de l'écran

**Problème.** `action_of`, `state_tone` et le texte du pied de page ne sont exercés
qu'indirectement, par un test de rendu qui n'affirme presque rien.

**Changement.** Aucun, sauf rendre ces aides visibles au module de tests.

**Fichiers.** `crates/ui/src/screen.rs`

**Tests à ajouter** :

| Test | Assertion |
|---|---|
| `offers_install_for_a_missing_app` | libellé « INSTALLER » et `Request::Install(id)` |
| `offers_update_only_when_the_release_is_newer` | libellé « METTRE À JOUR » et `Request::Update(id)` |
| `offers_open_when_up_to_date_or_unknown` | `Request::Open(id)` dans les deux cas |
| `colours_each_state_distinctly` | quatre tonalités, quatre couleurs distinctes |
| `footer_names_each_check_state` | les quatre textes, l'échec portant son motif |

**Fini quand.** les quatre états et les quatre `Check` sont couverts, `cargo xtask ci` vert.

---

## Étape 4 — Tester les états de l'écran

**Problème.** Une liste vide, une vérification en cours et un échec ne sont jamais rendus
en test : c'est pourtant là que l'interface peut mentir.

**Fichiers.** `crates/ui/src/screen.rs` (module `tests`)

**Tests à ajouter** (`egui_kittest`, interrogation par libellé) :

| Test | Assertion |
|---|---|
| `shows_a_loading_line_while_checking` | la ligne « Récupération des releases… » est présente |
| `shows_the_failure_reason` | le motif passé à `Update::Failed` apparaît dans le pied de page |
| `hides_the_unknown_app_when_searching` | filtre sans résultat → message d'absence |

**Fini quand.** trois tests de rendu, aucun sur la géométrie, `cargo xtask ci` vert.

---

## Étape 5 — Garde-fou sur le manifeste livré

**Problème.** `apps.json` est embarqué à la compilation ; une faute de frappe dans un dépôt
ne serait vue qu'au lancement.

**Fichiers.** `crates/ui/src/lib.rs` (ou `tests/`)

**Tests à ajouter** :

| Test | Assertion |
|---|---|
| `the_shipped_manifest_is_valid` | `manifest()` réussit et contient au moins une application |
| `every_application_has_an_asset_pattern` | le motif rend un nom non vide pour la plateforme courante |

**Fini quand.** le manifeste réel est couvert, `cargo xtask ci` vert.

---

## Étape 6 — Consigner ce qui reste manuel

**Problème.** Le registre réel et l'API réelle ne sont pas testables automatiquement ; sans
consigne écrite, personne ne les vérifie.

**Changement.** Un court runbook `docs/runbooks/verification-manuelle.md` : installer aNkou
en 1.0.0, lancer le launcher, vérifier la version détectée et la date publiée ; couper le
réseau, vérifier que l'échec s'affiche avec son motif.

**Fini quand.** le runbook existe et a été suivi une fois, la date et le résultat étant
notés dans la réponse de livraison.

---

## Étape 7 — Fermer les deux trous de la CI

**Problème.** La CI ne vérifie ni le verrou de dépendances ni la version minimale de Rust.
Un `Cargo.lock` désynchronisé, ou un appel à une API trop récente, passe aujourd'hui sans
bruit.

**Changement.**

- `xtask` : `--locked` ajouté aux étapes Compilation (`clippy`) et Tests ;
- workflow : job `msrv` qui compile avec la version exacte de `rust-version` (1.95) ;
- workflow : `concurrency` avec annulation de l'exécution obsolète.

**Fichiers.** `xtask/src/main.rs`, `.github/workflows/ci.yml`

**Vérification.** `cargo xtask ci` vert en local, puis le job `msrv` vert sur la PR. Le
verrou est vérifié en le désynchronisant volontairement une fois — la CI doit refuser.

**Fini quand.** les trois contrôles sont visibles dans l'exécution de la PR.

---

## Étape 8 — Produire le binaire en artefact

**Problème.** Rien ne prouve qu'un binaire se construit sur les trois systèmes autrement
qu'en compilant les tests ; aucune interface modifiée n'est essayable sans compiler.

**Changement.** Job `build` : `cargo build --release -p lpdnsoft-launcher --locked` sur la
matrice, puis `actions/upload-artifact` avec `lpdnsoft-launcher-<os>`.

**Fichiers.** `.github/workflows/ci.yml`

**Vérification.** L'artefact est téléchargeable depuis l'exécution de la PR et se lance sur
Windows.

**Fini quand.** trois artefacts existent au vert, aucun au rouge.

---

## Étape 9 — Décider de la release, plus tard

**Problème.** Le launcher n'a pas de chaîne de publication, alors qu'il devra se distribuer
lui-même le jour où il installera quoi que ce soit.

**Changement.** Aucun code. Une entrée dans l'ADR 0002 ou un ADR 0003 au moment de la
première fonction d'installation : formats, nommage, empreintes, essai d'installation.

**Fini quand.** la décision est écrite **avant** le premier paquet produit, pas après.

---

## Récapitulatif

| Étape | Tests ajoutés | Nature | Risque |
|---|---|---|---|
| 1 | 5 | contrat, pur | faible |
| 2 | 4 | intégration, pur | moyen : touche le pont |
| 3 | 5 | unitaire | faible |
| 4 | 3 | interface | faible (libellés) |
| 5 | 2 | garde-fou | faible |
| 6 | 0 | documentation | — |
| 7 | 0 | CI/CD : verrous | faible |
| 8 | 0 | CI/CD : artefacts | moyen : temps et espace CI |
| 9 | 0 | décision écrite | — |
| **Total** | **19** | | **26 → 45 tests** |

Ordre imposé : 1 et 2 changent du code, 3 à 5 n'ajoutent que des tests. Si le temps manque,
l'étape 2 est la plus utile — c'est là que se décide ce que l'utilisateur voit quand une
source tombe.
