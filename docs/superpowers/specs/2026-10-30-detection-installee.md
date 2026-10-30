# Spécification — détection de la version installée hors Windows

- **Date :** 2026-10-30
- **Statut :** proposée
- **Portée :** `crates/system`, `crates/core` (manifeste), `apps.json`
- **Voisins :** ADR 0002 (données réelles), ADR 0003 (cache)

## 1. Problème

Sous Windows, `crates/system` lit la version installée dans le registre de désinstallation.
Partout ailleurs, `installed_version` renvoie `None`, et l'interface en conclut **« NON
INSTALLÉE »** — ce qui est faux dès qu'un utilisateur a installé l'application par son `.deb`,
son `.rpm` ou son `.pkg`. Une valeur inventée serait grave ; une valeur **fausse** l'est
autant : elle pousse à réinstaller ce qui est déjà là.

## 2. Ce qu'on veut savoir, et ce qu'on ne peut pas savoir

| Question | Réponse |
|---|---|
| Le paquet est-il enregistré auprès du gestionnaire de paquets ? | Oui, par une requête locale |
| Quelle version annonce-t-il ? | Oui, mais dans un format propre à chaque outil |
| L'application est-elle installée autrement (archive, binaire copié, Flatpak) ? | **Non** — hors périmètre, et l'interface affichera « version inconnue » |

Cette limite est assumée : le launcher est fait pour installer par paquet, pas pour scanner
le disque.

## 3. Décision

1. **Une commande par famille de paquets, jamais de shell.** On lance le programme
   directement (`Command::args`), donc un nom de paquet ne peut pas être interprété comme
   une option ou une commande. Les noms viennent de `apps.json`, embarqué, pas d'une saisie.

2. **Linux : `dpkg-query`, puis `rpm -q`.** Debian et dérivés d'un côté, Fedora/RHEL/openSUSE
   de l'autre. Les deux peuvent coexister (rpm sur Debian) ; on essaie dans cet ordre et on
   retient la première réponse exploitable.

   | Outil | Commande | Sortie |
   |---|---|---|
   | dpkg | `dpkg-query -W -f=${Version}\n <paquet>` | `1.0.1`, ou `1:1.0.1-2` avec époque et révision |
   | rpm | `rpm -q --qf %{VERSION}\n <paquet>` | `1.0.1`, sans révision de paquet |

3. **macOS : `pkgutil`.** Un `.pkg` s'inscrit sous un identifiant inverse (`fr.lpdnsoft.ankou`).

   ```
   $ pkgutil --pkg-info fr.lpdnsoft.ankou
   package-id: fr.lpdnsoft.ankou
   version: 1.0.1
   ...
   ```

4. **Le nom de paquet vient du manifeste**, par plateforme, avec repli sur l'identifiant :

   ```json
   "packages": { "deb": "ankou", "rpm": "ankou", "pkg": "fr.lpdnsoft.ankou" }
   ```

   Un champ absent retombe sur `id`, par trois accesseurs (`deb_package`, `rpm_package`,
   `pkg_id`) : Linux a deux noms possibles, les fondre en un seul serait ambigu. C'est ce qui
   permettra à une application dont le paquet ne porte pas le même nom que son identifiant de
   rester déclarative.

5. **L'analyse des sorties est pure et testée.** `dpkg`, `rpm` et `pkgutil` ont chacun un
   analyseur (`version_from_deb`, `version_from_rpm`, `version_from_pkgutil`) qui ne fait
   aucune E/S. La normalisation est commune :

   - retirer l'époque Debian (`1:`) ;
   - retirer la révision de paquet (`-2`, `-1ubuntu3`) ;
   - retirer un préfixe `v` ;
   - garder les trois premiers nombres ; refuse moins de trois ;
   - **refuser une préversion** : un `~` juste après la suite (`1.0.1~rc1`) signifie une
     version non finale. La prendre pour `1.0.1` ferait croire à une version que
     l'utilisateur n'a pas — le même refus que dans `Version::from_str` ailleurs.

6. **L'exécution est injectable.** Un trait `Runner` rend la séquence « dpkg échoue → rpm
   répond », « les deux échouent », « sortie illisible » testables sur n'importe quelle
   machine, avec un faux exécutant. La vraie implémentation reste dix lignes.

7. **Pas de délai d'expiration.** Ces requêtes lisent une base locale et ne prennent pas de
   verrou (`-W`), contrairement à une installation. Elles tournent sur le fil de
   vérification, jamais sur celui de l'interface : un blocage ne gèle pas la fenêtre.
   Ajouter un délai demanderait de tuer un processus enfant, pour un risque qui n'existe pas
   ici.

## 4. Ce qui est testé, et comment

| Élément | Test | Sans machine Linux/macOS ? |
|---|---|---|
| Analyse `dpkg` (époque, révision, `v`) | direct | oui |
| Analyse `rpm` | direct | oui |
| Analyse `pkgutil` (ligne `version:`) | direct | oui |
| Sélection dpkg → rpm quand dpkg échoue | faux `Runner` | oui |
| Aucun outil disponible → `None` | faux `Runner` | oui |
| Sortie illisible → `None`, jamais une version inventée | faux `Runner` | oui |
| Commande réelle | vérification manuelle sur un conteneur Debian et un Fedora | non |

## 5. Risques

| # | Risque | Mitigation |
|---|---|---|
| R1 | Un nom de paquet mal saisi dans le manifeste | il vient du dépôt, relu en revue ; le repli sur `id` limite les dégâts |
| R2 | Une version Debian avec époque (`1:2.0.0-1`) | normalisation testée sur ces formes |
| R3 | Deux gestionnaires présents, réponses divergentes | ordre fixe documenté : dpkg d'abord |
| R4 | Le paquet est installé mais la commande manque | `None` → « version inconnue », jamais « non installée » : c'est déjà le comportement de `State::Unknown` |
