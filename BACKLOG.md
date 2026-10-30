# Cahier des tâches — launcher LPDNSoft

- **Dernière mise à jour :** 2026-10-30
- **Version courante :** 0.4.0 (branche en revue) — `main` est en 0.3.0
- **Décisions :** `docs/adr/` · **Stratégie de tests :** `docs/superpowers/specs/`

## Comment ça marche

| Statut | Sens |
|---|---|
| **Fait** | dans `main`, avec ses tests, et `cargo xtask ci` vert |
| **En cours** | branche ouverte, pas encore fusionnée |
| **À faire** | décidé, pas commencé |
| **Bloqué** | dépend d'une décision ou d'une ressource extérieure |

Règles de tenue :

1. **Chaque changement met ce fichier à jour**, dans le même commit que le code. Une tâche
   terminée passe en **Fait** avec son numéro de PR ; une tâche commencée passe en **En cours**.
2. Une tâche **À faire** porte son effort estimé et ses prérequis. Sans ça, elle n'est pas
   prête à être prise.
3. Une tâche **Bloquée** dit *pourquoi* et *ce qui la débloquerait* — jamais seulement « bloqué ».
4. Une tâche qui n'est plus voulue passe en **Abandonnée** avec sa raison, et ne disparaît
   pas : l'historique d'un refus vaut celui d'une livraison.

## Fait

| Tâche | Version | PR | Ce que ça couvre |
|---|---|---|---|
| Squelette : catalogue, interface, i18n, CI | 0.1 | #1 | 26 tests, trois OS, `cargo deny` |
| Données réelles : releases GitHub + registre Windows | 0.2 | #2 | 37 tests, ADR 0002 |
| Stratégie de tests, verrous CI (`--locked`, MSRV), artefacts | — | #3, #4 | 46 tests, 19 cibles |
| Licence MIT seule, avec un test garde-fou | — | #5 | déclaration = fichiers livrés |
| Actions honnêtes : boutons désactivés, liens actifs | — | #6 | 46 tests, refus du non-`https` |
| **Cache `ETag` et disque, repli hors ligne** | **0.3** | **#7** | **55 tests, ADR 0003** |
| Télécharger et vérifier `SHA256SUMS` (P1) | 0.4 | #9 | reprise `Range`, empreinte, publication atomique, 12 tests dont 5 sur serveur HTTP local |
| Détection Linux et macOS (P2) | 0.4 | #10 | `dpkg-query`, `rpm -q`, `pkgutil`, noms de paquet au manifeste, 15 tests |
| **Installer et mettre à jour (P3)** | **0.4** | **#11, #12, #13** | **plan, élévation, exécution, branchement UI, installation réelle vérifiée sous Linux** |
| **Ouvrir l'application installée (P4)** | **0.4** | **#14** | **chemin par système, bouton principal et entrée de menu, 8 tests** |

## En cours

| Tâche | État | Où |
|---|---|---|
| **P5 — Toutes les versions et notes de version** | terminée sur branche : notes lues dans la release (aucune requête), historique chargé à la demande, fenêtre de détail, 10 tests | `crates/core`, `crates/github`, `crates/ui` |
| **P6 — Modèle de PR et CONTRIBUTING** | terminée sur branche : un modèle et un guide par dépôt (launcher, ankou, lpdnsoft-libs) | trois dépôts |

## À faire

| Priorité | Tâche | Effort | Prérequis |
|---|---|---|---|
| **P3 fin** | Vérifier l'installation réelle sur Windows et macOS (Linux est fait) | ~1 h | un poste Windows élevable et un Mac |
| **P7** | Release du launcher : paquets, `SHA256SUMS`, publication | 1-2 j | P3, et la signature si certificats |

## Bloqué

| Tâche | Pourquoi | Ce qui la débloquerait |
|---|---|---|
| Signature de code Windows et macOS | Aucun certificat | Un certificat Apple Developer et un certificat de signature Windows |

En attendant, SmartScreen et Gatekeeper avertiront au premier lancement — les runbooks
d'installation devront le dire.

## Décisions à prendre

| Sujet | Pourquoi maintenant | Où |
|---|---|---|
| **Modèle d'élévation** : UAC à chaque installation, ou helper privilégié comme `ankoud` | **tranché** : élévation par installation, sans `unsafe` ni helper | ADR 0005, accepté |
| Durée de vie du cache | Aujourd'hui seule la réponse 304 décide ; une release retirée reste visible (ADR 0003 §5) | à trancher si le besoin apparaît |

## Ce qu'on ne fera pas, et pourquoi

| Idée | Raison |
|---|---|
| Une deuxième application factice dans `apps.json` | Sans dépôt réel, elle afficherait des lignes vides : le launcher perd sa seule qualité, l'honnêteté |
| AppImage pour le launcher | Un service système ne s'installe pas depuis une image portable (même raison qu'ADR 0011 d'aNKou) |
| Téléchargement automatique des mises à jour du launcher | Sans signature, cela revient à exécuter un binaire non vérifié (ADR 0002, ADR 0003) |

## Journal

| Date | Événement |
|---|---|
| 2026-10-30 | 0.1 : squelette et catalogue de démonstration |
| 2026-10-30 | 0.2 : vraies données GitHub et registre, fin des données inventées |
| 2026-10-30 | Stratégie de tests et verrous CI ; tests réorganisés en unitaires et intégration |
| 2026-10-30 | 0.3 : cache conditionnel, repli hors ligne annoncé comme tel |
| 2026-10-30 | 0.4 : téléchargement vérifié (`Range`, `SHA256SUMS`, publication atomique), ADR 0004 |
| 2026-10-30 | 0.4 : détection Linux (`dpkg-query`, `rpm -q`) et macOS (`pkgutil`), noms de paquet dans le manifeste |
| 2026-10-30 | 0.4 : installation — plan par système, élévation `pkexec`/UAC/`osascript`, exécution, ADR 0005 |
| 2026-10-30 | 0.4 : ouverture de l'application installée, avec le chemin résolu par système |
| 2026-10-30 | PR #9 à #14 fusionnées : le backlog passe P1 à P4 en **Fait** |
| 2026-10-30 | 0.4 : notes de version lues dans la release, historique chargé à la demande |
