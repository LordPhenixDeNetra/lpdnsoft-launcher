# Spécification — tests unitaires du launcher

- **Date :** 2026-10-30
- **Statut :** proposée
- **Portée :** dépôt `lpdnsoft-launcher`
- **Voisins :** `docs/adr/0001-architecture.md`, `docs/adr/0002-donnees-reelles.md`

---

## 1. Objectif

Rendre chaque changement **vérifiable avant livraison**, sans réseau ni machine
préparée. Aujourd'hui 26 tests couvrent le cœur et les analyseurs ; l'orchestration, la
classification des erreurs HTTP et les états de l'interface ne sont pas testés. C'est
précisément là que les régressions coûteront cher, parce que ce sont les chemins qui
décident de ce que l'utilisateur voit.

Deux principes gouvernent tout ce document :

1. **Un test qui ne peut pas échouer ne prouve rien.** Un test nouveau doit échouer sur le
   code d'avant le changement.
2. **Ce qui n'est pas testable est mal découpé.** Quand une règle est difficile à tester,
   c'est le découpage qu'il faut corriger, pas le test qu'il faut abandonner.

## 2. Niveaux, cibles et outils

| Niveau | Cible | Outil | Ce qui est exclu |
|---|---|---|---|
| Unitaire pur | `lpdnsoft-launcher-core` : versions, motifs d'asset, manifeste, releases, catalogue, états | `#[cfg(test)]` — aucun I/O | rien : tout est déterministe |
| Contrat | `lpdnsoft-launcher-github` : lecture du JSON, classification des statuts HTTP | `#[cfg(test)]`, JSON en dur | le réseau |
| Contrat | `lpdnsoft-launcher-system` : `version_from_display` | `#[cfg(test)]` | le registre |
| Intégration | `bridge::check` : orchestrer deux sources, décider `Catalog` ou `Failed` | sources injectées | le réseau, le registre |
| Interface | `lpdnsoft-launcher-ui` : libellés, tonalités, pied de page, liste vide, menu | `egui_kittest` et fonctions pures | le rendu visuel exact |
| **Hors tests automatiques** | registre Windows réel, API GitHub réelle, installeurs | vérification manuelle, consignée | — |

Le dernier niveau est assumé : un test qui interroge le vrai registre dépendrait de la
machine, et un test qui interroge la vraie API dépendrait du réseau et d'un quota de
soixante requêtes par heure. Ces chemins sont vérifiés à la main, et la vérification est
écrite dans la réponse de livraison.

## 3. Règles d'écriture

1. **Un comportement, un test, un nom.** Le nom décrit le comportement attendu
   (`refuses_a_tag_that_is_not_a_version`), jamais le nom de la fonction testée.
2. **Tout `pub` du cœur a son test**, y compris les cas limites : chaîne vide, valeur
   absente, identifiant inconnu.
3. **Chaque variante d'erreur a un test qui la produit.** Une erreur jamais construite en
   test est une erreur jamais relue.
4. **Aucune dépendance au réseau, à l'heure, au fuseau, à la locale ou à l'ordre des
   fichiers.** Les dates sont des constantes, les locales sont passées explicitement.
5. **Aucun `sleep`.** On attend une condition avec une échéance, jamais une durée fixe.
6. **Les tests d'interface interrogent le sens** — un libellé, un état — jamais une
   position en pixels.
7. **Un test ignoré porte son motif** : `#[ignore = "PostgreSQL requis"]`, comme ailleurs
   dans la suite.
8. **Un test existant ne se désactive pas** pour faire passer un changement : on le
   corrige, ou on explique dans la revue pourquoi il n'a plus de sens.

## 4. Définition de « terminé »

Un travail n'est livré que si les cinq points sont vrais :

- [ ] Le comportement ajouté ou modifié a **au moins un test** qui échoue sans lui.
- [ ] `cargo xtask ci` sort en **exit 0** : i18n complet, `fmt --check`,
      `clippy -D warnings`, tests, `cargo deny check`.
- [ ] Les cas d'erreur du chemin touché sont couverts, pas seulement le cas heureux.
- [ ] La réponse de livraison indique **la commande lancée et son résultat exact**.
- [ ] Les commandes git et le formulaire de PR accompagnent la livraison (règle locale).

## 5. Couverture : ce qu'on mesure, et ce qu'on ne mesure pas

Aucun pourcentage n'est visé. Un objectif de couverture se contourne en écrivant des tests
qui n'affirment rien, et il ne dit rien de la qualité des assertions.

On mesure des **invariants**, vérifiables à la relecture :

| Invariant | Vérification |
|---|---|
| Tout `pub` du cœur est exercé | revue : un test par fonction publique |
| Toute variante de `State` est atteinte | `catalog::tests` construit les quatre |
| Toute variante d'erreur est produite | un test par variante, dans son crate |
| Toute transition de `Check` est atteinte | `screen::tests::records_the_check_state` |
| Aucun test ne touche le réseau | revue : aucun `reqwest` hors de `github` |

## 6. Ce qui reste hors périmètre

- **Le registre Windows** : la lecture dépend de la machine. Seule la transformation
  `DisplayVersion → Version` est testée ; le parcours des clés est vérifié à la main.
- **L'API GitHub** : quota, coupures, 404. Seule la classification des statuts et la
  lecture du JSON sont testées.
- **L'installation** : elle n'existe pas encore. Ses tests viendront avec elle, avant elle.
- **Le rendu visuel** : `egui_kittest` vérifie la présence et le texte des éléments, pas
  l'esthétique. Une capture d'écran reste la preuve visuelle.

## 7. Risques

| # | Risque | Mitigation |
|---|---|---|
| R1 | Tests d'interface fragiles, cassés à chaque retouche de mise en page | interroger les libellés, jamais la géométrie |
| R2 | Tests qui passent pour de mauvaises raisons (assertion vide, `is_ok()` seul) | assertions sur la valeur, pas sur l'absence d'erreur |
| R3 | Dérive entre le plan et le code | chaque étape du plan se termine par `cargo xtask ci` vert |
| R4 | Le pont devient intestable en grandissant | garder `check` mince, les décisions dans le cœur |
