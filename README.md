# lpdnsoft-launcher

Launcher de la suite **LPDNSoft** : il liste les applications de la suite, affiche leur version
installée et la dernière publiée, et les installe ou les met à jour depuis leurs releases GitHub.

Référence visuelle et fonctionnelle : le launcher ArtCraft (cité par la spec d'aNkou §1).

![Le launcher en v0.2, sur données réelles](docs/capture.png)

## État — v0.2 (données réelles)

Ce qui marche :

- **Vraies releases** : une requête `GET /releases/latest` par application, sans intermédiaire
  ni jeton — version disponible et date de publication viennent de GitHub ;
- **Vraie version installée** sous Windows, lue dans les clés de désinstallation du registre ;
- **États calculés** : à jour, mise à jour disponible, non installée, version inconnue ;
- **Interface complète** : pastilles d'état, recherche, menu par application, deux langues ;
- **Pied de page honnête** : jamais vérifié / vérification en cours / vérifié / échec motivé ;
- 26 tests, `fmt` et `clippy -D warnings` propres, `cargo deny` sans avis.

Ce qui n'existe pas encore :

- aucun téléchargement, aucune vérification `SHA256SUMS`, aucune installation ni mise à jour ;
- les boutons Installer / Mettre à jour / Ouvrir et le menu remplissent une file de `Request`
  que rien ne consomme ;
- aucune détection hors Windows, aucun cache disque : un lancement hors ligne n'affiche
  d'abord qu'un échec motivé, puis les données de la passe précédente s'il y en a une.

## Compiler et lancer

Prérequis : Rust stable ≥ 1.95 et le dépôt `lpdnsoft-libs` cloné **à côté** de celui-ci.

```sh
cargo run -p lpdnsoft-launcher            # langue du système
cargo run -p lpdnsoft-launcher -- --lang fr
cargo xtask ci                             # fmt, clippy, tests, cargo deny
```

## Architecture

```
crates/core   catalogue, versions, états — aucune E/S
crates/ui     interface egui (thème lpdnsoft-ui), données de démonstration
apps/launcher binaire : arguments, journal, lancement de la fenêtre
```

Décisions et limites : `docs/adr/0001-architecture.md`.

Licence : MIT. Le texte est dans `LICENSE-MIT` ; le manifeste le déclare, et un test
vérifie que la déclaration et les fichiers livrés ne divergent pas.
