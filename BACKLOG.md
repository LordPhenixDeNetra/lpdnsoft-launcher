# Cahier des tâches — launcher LPDNSoft

- **Dernière mise à jour :** 2026-10-30
- **Version courante :** 0.3.0
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

## En cours

*Aucune.* La PR #7 attend sa revue.

## À faire

| Priorité | Tâche | Effort | Prérequis |
|---|---|---|---|
| **P1** | **Télécharger le paquet et vérifier `SHA256SUMS`** — parseur d'empreintes, calcul, refus d'un fichier qui ne correspond pas, reprise après interruption | ~3 h | — |
| **P2** | Détection des versions installées sous Linux (`dpkg-query`, `rpm -q`) et macOS (`pkgutil`) | ~3 h | — |
| **P3** | Installer et mettre à jour : MSI, `.deb`/`.rpm`, `.pkg`, élévation | 2-3 j | **ADR 0004** (modèle d'élévation) |
| **P4** | Ouvrir l'application installée | ~2 h | chemin d'installation lu par OS |
| **P5** | Toutes les versions et notes de version | ~4 h | conserver les anciennes releases |
| **P6** | `pull_request_template.md` et `CONTRIBUTING.md`, dans les trois dépôts | ~30 min | — |
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
| **Modèle d'élévation** : UAC à chaque installation, ou helper privilégié comme `ankoud` | Il conditionne l'architecture de P3 | ADR 0004, à écrire avant d'implémenter |
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
