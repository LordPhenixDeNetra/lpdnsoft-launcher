# ADR 0002 — Lire les vraies données : releases GitHub et version installée

- Statut : accepté
- Date : 2026-10-30
- Complète : ADR 0001

## Contexte

La v0.1 affichait un catalogue de démonstration : versions, dates et applications inventées.
Elle affichait même « vérifié à l'instant · via GitHub » sans avoir interrogé quoi que ce soit.
Une interface qui ment sur son comportement est pire qu'une interface vide.

## Décision

1. **Les releases viennent de GitHub, sans intermédiaire.** Une requête par application :
   `GET https://api.github.com/repos/<propriétaire>/<nom>/releases/latest`, avec l'agent
   `lpdnsoft-launcher/<version>` exigé par l'API. Ni serveur, ni jeton.

2. **La liste des applications est un fichier versionné**, `apps.json`, embarqué à la
   compilation et validé au démarrage (identifiants uniques, dépôt `propriétaire/nom`).
   Ajouter une application reste une entrée de configuration.

3. **La version installée se lit sur le poste.** Windows : les clés de désinstallation du
   registre (`HKLM` et `HKCU`, vues 64 et 32 bits), où l'installeur MSI inscrit
   `DisplayName` et `DisplayVersion`. Ailleurs, aucune détection pour l'instant : la ligne
   affiche une version inconnue plutôt qu'une valeur inventée.

4. **Une vérification ne bloque jamais l'interface.** Un fil porte un runtime tokio ; il
   pousse `Checking`, puis `Catalog` ou `Failed`, et réveille la fenêtre. Le bouton
   « Actualiser » redemande une passe.

5. **L'état de la vérification est affiché honnêtement** : jamais vérifié, en cours, vérifié,
   ou l'échec avec son motif. Un échec total ne laisse pas un catalogue vide et muet.

6. **Les actions qui n'existent pas encore n'en prétendent pas moins.** Installer, mettre à
   jour, ouvrir et les entrées du menu remplissent toujours une file de requêtes que rien
   ne consomme. Le téléchargement, la vérification `SHA256SUMS` et l'installation restent
   à écrire.

## Conséquences

- **Soixante requêtes par heure** au total, sans authentification. Deux applications tiennent
  largement ; dix applications demandent un cache `ETag` (une réponse 304 ne consomme pas de
  quota) avant toute mise en service.
- **La première requête part au lancement.** Hors ligne, l'application affiche l'échec et le
  motif, sans bloquer : c'est le comportement attendu, pas un défaut.
- **Le registre n'est qu'un début.** Une application installée autrement que par son MSI
  (archive décompressée, paquet d'une autre distribution) restera invisible. La détection
  Linux (`dpkg-query`, `rpm -q`) et macOS (`pkgutil`) viendra avec les installeurs.
- **Aucun cache disque.** Un lancement hors ligne ne montre rien ; le cache des dernières
  réponses est la prochaine brique utile.
