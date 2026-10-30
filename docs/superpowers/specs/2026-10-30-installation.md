# Spécification — installer et mettre à jour

- **Date :** 2026-10-30
- **Statut :** proposée
- **Portée :** `crates/install`, à brancher ensuite sur l'interface
- **Décision d'architecture :** ADR 0005 (élévation)

## 1. Problème

Le launcher sait vérifier (ADR 0002), mettre en cache (ADR 0003) et télécharger un paquet
vérifié (ADR 0004). Il ne sait pas l'installer, donc il ne fait encore rien pour l'utilisateur.

## 2. Ce qu'« installer » veut dire, par système

| Système | Format | Commande de base | Après installation |
|---|---|---|---|
| Windows | `.msi` | `msiexec /i <fichier> /qn /norestart` | service enregistré, entrée de désinstallation créée |
| Linux Debian | `.deb` | `dpkg -i <fichier>` | paquet enregistré dans dpkg |
| Linux RPM | `.rpm` | `rpm -U <fichier>` | paquet enregistré dans rpm |
| macOS | `.pkg` | `installer -pkg <fichier> -target /` | reçu `pkgutil`, démon launchd |

**La mise à jour est une installation.** On télécharge la nouvelle version et on lance le même
installeur : les gestionnaires remplacent la version précédente. Il n'y a donc pas de code
« mise à jour » distinct — seulement une version différente dans le nom du fichier.

## 3. Décisions

1. **Le plan est pur.** `plan(paquet, système)` rend `programme`, `arguments`, `summary` et
   l'indication d'élévation. Aucune E/S : on peut afficher la commande avant de la lancer,
   et la tester sans rien exécuter.

2. **L'élévation enveloppe, elle ne remplace pas.** `elevated(plan, système)` produit la
   commande finale : `pkexec …`, `powershell … -Verb RunAs …`, `osascript … with
   administrator privileges`. La commande de base reste lisible et testable à part.

3. **Un chemin dangereux est refusé.** Le chemin finit dans une ligne PowerShell ou
   AppleScript. S'il contient `"`, `'`, `` ` ``, `$`, `;`, `|`, `&`, `\n` ou `\r`, le plan
   renvoie `Refused` : pas d'échappement artisanale, pas de surprise.

4. **Extension et système doivent correspondre.** Un `.deb` sur Windows est `Unsupported` :
   mieux vaut refuser que lancer une commande qui échouera de travers.

5. **L'exécution est mince et injectable.** `Executor` lance le plan ; les tests utilisent un
   faux exécutant pour vérifier ce qui aurait été lancé, sans installer quoi que ce soit.

6. **Aucun redémarrage implicite.** `/norestart` est toujours passé ; le launcher ne redémarre
   jamais la machine.

## 4. Codes de sortie

| Situation | Ce que voit l'utilisateur |
|---|---|
| Code 0 | installation réussie, la ligne passera « à jour » à la vérification suivante |
| Code ≠ 0 | le code exact et la sortie d'erreur de l'outil, sans interprétation |
| `pkexec` absent | échec explicite avec la commande à lancer à la main |
| Invite d'élévation refusée | échec explicite, aucune tentative de contournement |

## 5. Ce qui est testé

| Élément | Test |
|---|---|
| Plan par système et par extension | direct |
| Extension incompatible → `Unsupported` | direct |
| Chemin contenant `'`, `"`, `$`, `;`, saut de ligne → `Refused` | direct |
| Enveloppe d'élévation (programme et arguments) | direct |
| Ce qui serait lancé, sans rien lancer | faux `Executor` |
| Installation réelle | manuelle, sur les trois systèmes (étape 5 du plan) |

## 6. Risques

| # | Risque | Mitigation |
|---|---|---|
| R1 | Un installeur échoue à mi-parcours et laisse un paquet cassé | pas de rollback en v1 : le code et la sortie sont relayés, l'utilisateur décide (ADR 0005 §6) |
| R2 | `pkexec` ou `powershell` absent | échec explicite, jamais silencieux |
| R3 | Un chemin piégé injecte une commande | refus des caractères dangereux, testé |
| R4 | L'utilisateur refuse l'invite | aucun contournement, échec net |
