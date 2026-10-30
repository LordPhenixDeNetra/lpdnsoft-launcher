# lpdnsoft-launcher

Launcher de la suite **LPDNSoft** : il liste les applications de la suite, affiche leur version
installée et la dernière publiée, et les installe ou les met à jour depuis leurs releases GitHub.

Référence visuelle et fonctionnelle : le launcher ArtCraft (cité par la spec d'aNkou §1).

![Le launcher en v0.1, sur données de démonstration](docs/capture.png)

## État — v0.1 (socle)

Ce qui marche aujourd'hui :

- **Catalogue typé** : applications, plateformes, motifs de nom d'asset, releases, états ;
- **Interface complète** : liste, pastilles d'état, versions, dates, recherche, actions et menu ;
- **i18n** anglais/français, clés vérifiées à la compilation ;
- 20 tests, `fmt` et `clippy -D warnings` propres.

Ce qui n'est pas encore là, volontairement :

- aucun accès réseau : le catalogue vient de `crates/ui/src/fixture.rs` ;
- aucun téléchargement, aucune vérification `SHA256SUMS`, aucune installation ;
- aucune détection des versions réellement installées ;
- les boutons remplissent une file de `Request`, que rien ne consomme encore.

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

Licence : MIT ou Apache-2.0, au choix.
