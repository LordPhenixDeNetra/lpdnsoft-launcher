# Spécification — toutes les versions et les notes de version

- **Date :** 2026-10-30
- **Statut :** proposée
- **Portée :** `crates/core` (modèle), `crates/github` (API), `crates/ui` (fenêtre de détail)
- **Voisins :** ADR 0002 (données réelles), ADR 0003 (cache)

## 1. Problème

La liste affiche une seule ligne par application : la version publiée. Deux questions restent
sans réponse, et ce sont celles qu'on se pose avant d'installer :

1. **Que contient cette version ?** Les notes de version sont publiées par GitHub, et le
   launcher les ignore — il se contente d'un lien vers la page.
2. **Que contenait la précédente ?** L'historique existe, dans `/releases`, et n'est pas lu.

## 2. Ce qu'on veut

| Action | Ce que l'utilisateur obtient | Coût réseau |
|---|---|---|
| Notes de la version publiée | le texte de la release, dans une fenêtre, sans quitter le launcher | **aucun** : le corps est déjà dans la réponse `/releases/latest`, mise en cache |
| Toutes les versions | la liste des versions publiées, avec date et notes | **une** requête par application, au moment où on la demande |

## 3. Décisions

1. **Les notes font partie de la release.** `Release` reçoit `notes: Option<String>`, lue dans
   le champ `body` de l'API. Elles sont donc mises en cache avec la release (ADR 0003), et
   afficher les notes de la version courante ne coûte **rien**.

2. **La liste des versions se demande, elle ne se précharge pas.** Une requête par
   application, seulement si l'utilisateur ouvre la fenêtre. Précharger l'historique de toutes
   les applications consommerait le quota (soixante par heure) pour une information rarement
   regardée.

3. **Une seule fenêtre, deux provenances.** Le détail s'ouvre sur les notes de la version
   publiée, ou sur l'historique complet ; c'est le même affichage, avec une liste quand elle
   est chargée.

4. **Le texte est affiché tel quel, jamais interprété.** Pas de rendu Markdown : on montre ce
   que l'auteur a écrit. Un rendu approximatif serait pire qu'un texte brut honnête.

5. **`ChooseVersion` reste désactivé.** Choisir une version *à installer* suppose de savoir
   l'installer, donc d'avoir la release dans l'historique avec son actif — c'est la suite,
   pas cette tâche. Le menu garde trois entrées utiles : notes, historique, masquer.

## 4. États

| État | Affichage |
|---|---|
| Notes disponibles | fenêtre avec le texte, titre = version |
| Notes absentes (l'auteur n'a rien écrit) | fenêtre avec « aucune note pour cette version » |
| Historique en cours | fenêtre avec « chargement… » |
| Historique chargé | liste version + date, et le texte de la sélection |
| Échec réseau | la raison exacte, pas une liste vide |

## 5. Ce qui est testé

| Élément | Test |
|---|---|
| `body` lu et normalisé (chaîne vide → `None`) | direct, sur du JSON réel |
| `/releases` : liste, ordre, champs manquants | direct, sur du JSON réel |
| Ancien cache sans `notes` reste lisible | direct |
| Ouverture de la fenêtre depuis les notes | écran |
| Notes / historique indisponibles sans release | liens |

## 6. Risques

| # | Risque | Mitigation |
|---|---|---|
| R1 | Une release sans `body` casse l'affichage | `Option`, et un message explicite |
| R2 | Un historique long noie la fenêtre | `per_page=30`, et une zone défilante |
| R3 | Le quota est entamé par inadvertance | la liste n'est demandée qu'à l'ouverture, jamais au rafraîchissement |
