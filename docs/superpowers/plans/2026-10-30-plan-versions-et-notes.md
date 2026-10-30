# Plan — toutes les versions et les notes de version

- **Date :** 2026-10-30
- **Spécification :** `docs/superpowers/specs/2026-10-30-versions-et-notes.md`
- **Règle :** chaque étape se termine par `cargo xtask ci` en exit 0.

---

## Étape 1 — Les notes entrent dans le modèle

**Changement.** `Release.notes: Option<String>`, alimenté par le champ `body` de l'API ;
chaîne vide ou blanche → `None`. Les littéraux des tests sont complétés.

**Tests.** `body` présent, absent, vide ; ancien cache sans le champ reste lisible.

**Fini quand.** la réponse réelle d'aNkou donne des notes non vides.

---

## Étape 2 — La liste des versions

**Changement.** `github::releases(client, app, limit)` : `GET /repos/<…>/releases?per_page=30`,
analyse de chaque élément avec la même fonction que la release courante.

**Tests.** deux releases, ordre conservé, élément sans `tag_name` ignoré, `body` absent
toléré.

**Fini quand.** la liste réelle d'aNkou rend 1.0.1 et 1.0.0.

---

## Étape 3 — La fenêtre de détail

**Changement.** `Screen.detail: Option<Detail>` ; `Detail { app, title, entries, loading }`.
`Request::Notes(id)` l'ouvre avec la version publiée ; `Request::AllVersions(id)` l'ouvre vide
puis la remplit (`Update::Versions`). Fermeture par bouton ou `Échap`.

**Tests.** ouverture depuis les notes, chargement puis remplissage, échec affiché, fermeture.

**Fini quand.** les deux entrées du menu fonctionnent dans l'application réelle.

---

## Étape 4 — Documentation et suivi

**Changement.** README, consignes, backlog (P5 → fait), capture si l'affichage change.

**Fini quand.** aucun document ne dit plus que les notes n'existent pas.

---

## Récapitulatif

| Étape | Nature | Tests |
|---|---|---|
| 1 | modèle | 3 |
| 2 | API | 4 |
| 3 | interface | 4 |
| 4 | documentation | — |
