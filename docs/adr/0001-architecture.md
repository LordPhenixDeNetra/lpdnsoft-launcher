# ADR 0001 — Architecture du launcher LPDNSoft

- Statut : accepté
- Date : 2026-10-30

## Contexte

La suite LPDNSoft compte une application (aNkou) et doit en accueillir d'autres. Une spec
ultérieure (celle d'aNkou, §2) prévoit un launcher « quand une 2ᵉ app existera », et la
convention de release d'aNkou (`ankou-<version>-<os>-<arch>.<ext>` + `SHA256SUMS`) a été
conçue pour qu'un launcher la lise (ADR 0011 d'aNkou).

Le modèle de référence est ArtCraft : une liste d'applications, leur état, l'installation et
la mise à jour en un clic, plus un menu par application (versions, notes, site, dépôt).

## Décision

1. **Dépôt séparé** `lpdnsoft-launcher`, même disposition que `ankou` : un workspace Cargo,
   dépendance par chemin vers `../lpdnsoft-libs/crates/<nom>` en développement.

2. **Cœur pur.** `lpdnsoft-launcher-core` ne fait aucune E/S : versions, manifeste, releases,
   états, filtrage. L'interface et le réseau viennent après, et le cœur reste testable sans
   réseau ni droits.

3. **Pas de serveur.** Le launcher lit les releases GitHub de chaque application, comme
   `ankou-api-client::update`. ArtCraft ajoute une API dédiée ; LPDNSoft s'en passe : moins
   d'infrastructure, et un launcher utilisable hors ligne sur des données en cache.

4. **Catalogue déclaratif** (`apps.json`) : identifiant, nom, sous-titre, auteur, dépôt,
   page d'accueil, motif d'asset, indicateur de masquage. Ajouter une application est une
   entrée de configuration, pas du code.

5. **v0.1 sans effet de bord.** Aucun téléchargement, aucune installation : l'interface
   s'appuie sur un catalogue de démonstration et remplit une file de `Request`. Le branchement
   réseau puis l'installation viennent ensuite, derrière cette file.

6. **Installation prévue** : télécharger l'asset de la plateforme, vérifier l'empreinte
   contre `SHA256SUMS`, puis lancer l'installateur du système (`.msi`, `.deb`/`.rpm`, `.pkg`).
   L'élévation est demandée par installation, jamais au lancement du launcher.

7. **Interface egui** avec `lpdnsoft-ui`, thème sombre par défaut comme la référence.

## Conséquences

- **Limite de l'API GitHub** : 60 requêtes par heure sans authentification. Le launcher devra
  mettre en cache les réponses et utiliser `ETag` (une réponse 304 ne consomme pas de quota) ;
  sans cela, une suite de neuf applications épuise le quota en quelques rafraîchissements.
- **Installateurs non signés** : un outil qui télécharge puis exécute des paquets déclenchera
  SmartScreen et Gatekeeper. À traiter avec la signature de code, pas avant.
- **Une seule application pour l'instant** : le launcher n'apporte, en v0.1, que la gestion
  des versions d'aNkou. Il devient réellement utile avec la deuxième application.
- La détection des versions installées est propre à chaque OS (registre ARP sous Windows,
  `dpkg-query`/`rpm -q`, `pkgutil`) : elle vivra dans un crate à côté du cœur, pas dedans.
