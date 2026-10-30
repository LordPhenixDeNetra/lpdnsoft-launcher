# Publier une version du launcher

- **Portée :** `lpdnsoft-launcher`
- **Décision :** ADR 0011 d'aNkou, reprise ici (paquets + `SHA256SUMS`)
- **Pipeline :** `.github/workflows/release.yml`

## Ce que la release doit produire

| Système | Fichier | Contenu |
|---|---|---|
| Windows | `lpdnsoft-launcher-<v>-windows-x86_64.msi` | `%ProgramFiles%\LPDNSoft Launcher\lpdnsoft-launcher.exe` |
| Linux | `lpdnsoft-launcher-<v>-linux-x86_64.deb` | `/usr/bin/lpdnsoft-launcher`, entrée de menu |
| Linux | `lpdnsoft-launcher-<v>-linux-x86_64.rpm` | idem, paquet RPM |
| macOS | `lpdnsoft-launcher-<v>-macos-aarch64.pkg` | `/Applications/LPDNSoft Launcher.app` |
| macOS | `lpdnsoft-launcher-<v>-macos-x86_64.pkg` | idem, Intel |
| tous | `SHA256SUMS` | une ligne par fichier |

Ces noms ne sont pas décoratifs : le launcher les cherche avec le motif d'`apps.json`
(`lpdnsoft-launcher-{version}-{os}-{arch}.{ext}`). `apps/launcher/tests/release_assets.rs`
vérifie que le workflow et les fichiers d'empaquetage les produisent bien.

## Étapes

1. **Monter la version** dans `Cargo.toml` (`[workspace.package] version`) et dans les
   dépendances internes.
2. **Vérifier** :

   ```
   cargo xtask ci
   ```

3. **Mettre à jour le suivi** : `BACKLOG.md` (tâche livrée avec son numéro de PR), et
   `README.md` si l'état change.
4. **Fusionner sur `main`.** C'est tout : le workflow `Release` se déclenche sur `main`,
   construit les cinq paquets, calcule `SHA256SUMS`, crée le tag `v<version>` et publie la
   release.

   Deux garde-fous dans ce déclenchement :
   - si une release `v<version>` existe déjà, le job `prepare` répond `release=false` et tout
     le reste est sauté — fusionner deux fois ne publie pas deux fois ;
   - `concurrency: release` empêche deux publications simultanées.

5. **Les notes de version sont générées par GitHub** (`--generate-notes`) à partir des pull
   requests fusionnées depuis la release précédente, avec la liste des commits et le lien de
   comparaison. Personne ne les rédige, donc elles ne peuvent pas devenir fausses — et le
   launcher les affiche (*Notes*, P5).

6. **En cas d'échec**, corriger puis relancer sans nouveau commit :

   ```
   gh workflow run release.yml
   ```

   Le job `prepare` revérifie l'absence de la release et repart.

7. **Vérifier la release** : la page GitHub doit lister six fichiers, dont `SHA256SUMS`.

## Vérifier un paquet publié

```
gh release download v0.4.0 -R LordPhenixDeNetra/lpdnsoft-launcher -p "SHA256SUMS" -D %TEMP%\lpdnsoft
gh release download v0.4.0 -R LordPhenixDeNetra/lpdnsoft-launcher -p "*.msi" -D %TEMP%\lpdnsoft
certutil -hashfile "%TEMP%\lpdnsoft\lpdnsoft-launcher-0.4.0-windows-x86_64.msi" SHA256
```

L'empreinte doit correspondre à la ligne du fichier dans `SHA256SUMS`.

Ensuite, installer, puis relancer le launcher : sa propre ligne doit passer de « MISE À JOUR
DISPONIBLE » à « À JOUR 0.4.0 ». C'est la vérification de bout en bout de la chaîne —
téléchargement, empreinte, installation, détection.

## Signature — état actuel

**Les paquets ne sont pas signés.** Aucun certificat n'est disponible (voir la section
*Bloqué* de `BACKLOG.md`) :

- Windows : SmartScreen affichera un avertissement au premier lancement ;
- macOS : Gatekeeper refusera le `.pkg` sans clic droit → *Ouvrir*, ou sans
  `xattr -d com.apple.quarantine`.

Ces avertissements sont **assumés et documentés**, pas contournés. Une fois un certificat
disponible, la signature s'ajoute dans le workflow, avant le calcul des empreintes.
