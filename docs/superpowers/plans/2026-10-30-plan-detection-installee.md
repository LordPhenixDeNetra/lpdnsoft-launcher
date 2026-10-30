# Plan — détection de la version installée hors Windows

- **Date :** 2026-10-30
- **Spécification :** `docs/superpowers/specs/2026-10-30-detection-installee.md`
- **Règle :** chaque étape se termine par `cargo xtask ci` en exit 0.

---

## Étape 1 — Le nom de paquet entre dans le manifeste

**Changement.** `AppSpec` reçoit `packages` (facultatif) : `deb`, `rpm`, `pkg` ; un champ
absent retombe sur `id`, via trois accesseurs (`deb_package`, `rpm_package`, `pkg_id`).

**Fichiers.** `crates/core/src/app.rs`, `apps.json`, `crates/core/tests/manifest.rs`

**Tests.** Un nom explicite est retenu ; un champ absent retombe sur `id` ; le manifeste livré
passe toujours la validation.

**Fini quand.** `manifest()` accepte les deux applications, l'`apps.json` livré déclare
`fr.lpdnsoft.ankou` pour macOS.

---

## Étape 2 — Les analyseurs, purs

**Changement.** Trois fonctions sans E/S dans `crates/system` :

| Fonction | Entrée | Sortie attendue |
|---|---|---|
| `version_from_deb` | `1:1.0.1-2ubuntu3` | `1.0.1` |
| `version_from_rpm` | `1.0.1` | `1.0.1` |
| `version_from_pkgutil` | sortie complète de `--pkg-info` | `1.0.1` (ligne `version:`) |

Toutes refusent ce qui n'a pas trois nombres et ne devinent jamais.

**Fichiers.** `crates/system/src/lib.rs`, `crates/system/tests/versions.rs`

**Tests.** Époque, révision, préfixe `v`, quatre composants, sortie vide, sortie d'erreur,
`pkgutil` sans ligne `version:`.

**Fini quand.** chaque forme réelle rencontrée est couverte, y compris les refus.

---

## Étape 3 — L'exécution devient injectable

**Changement.** Trait `Runner { fn output(&self, programme, arguments) -> Option<String> }`,
implémentation réelle (`Command`), et `detect(app, runner)` par plateforme :

- Linux : `dpkg-query`, puis `rpm` si le premier échoue ;
- macOS : `pkgutil` ;
- Windows : inchangé (registre).

**Fichiers.** `crates/system/src/lib.rs`, `crates/system/tests/detection.rs`

**Tests.** Avec un faux `Runner` : dpkg répond → sa version ; dpkg échoue, rpm répond → la
version de rpm ; les deux échouent → `None` ; sortie illisible → `None` ; commande absente →
`None`. Plus : la séquence d'appels est bien dpkg puis rpm, jamais l'inverse.

**Fini quand.** la sélection est prouvée sans machine Linux, et `installed_version` reste la
seule fonction appelée par le pont.

---

## Étape 4 — Documentation et suivi

**Changement.** `README.md`, `AGENTS.md`, `CLAUDE.md`, `BACKLOG.md` : P2 passe en *En cours*,
la détection n'est plus décrite comme « Windows seulement ».

**Fini quand.** aucun document n'affirme plus que la détection est limitée à Windows.

---

## Étape 5 — Vérification manuelle (hors CI)

Sur un conteneur Debian puis Fedora, installer `ankou` en 1.0.0 depuis son paquet, lancer le
launcher et vérifier la version affichée. Sur macOS, `pkgutil --pkg-info` sur un `.pkg` posé à
la main. Résultat consigné dans la réponse de livraison, avec la commande et sa sortie.

**Fini quand.** la sortie réelle des trois outils est comparée à ce que les analyseurs
attendent.

---

## Récapitulatif

| Étape | Nature | Tests ajoutés |
|---|---|---|
| 1 | manifeste | 3 |
| 2 | analyseurs purs | 8 |
| 3 | exécution injectable | 5 |
| 4 | documentation | — |
| 5 | vérification manuelle | — |
