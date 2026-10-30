# Vérification d'une installation réelle

- **Date :** 2026-10-30
- **Paquet :** aNkou `v1.0.1`, release publique du dépôt `LordPhenixDeNetra/ankou`
- **Étape du plan :** étape 5 de `docs/superpowers/plans/2026-10-30-plan-installation.md`

Ce document consigne ce qui a été **réellement exécuté**, et ce qui ne l'a pas été. Il ne
décrit pas ce qui devrait se passer : il rapporte ce qui s'est passé.

## Résultat

| Système | Installation réelle | Ce qui a été prouvé |
|---|---|---|
| Linux (Ubuntu 26.04.1, WSL) | **oui**, installée puis désinstallée | paquet, dépendances, service, version lue par la commande de détection |
| Windows 11 | **non** | la détection lit une vraie entrée de registre ; le MSI n'a pas été exécuté |
| macOS | **non** | rien, faute de machine |

## Linux — installation complète

Le `.deb` réel a d'abord été comparé à `SHA256SUMS` :

```
796f996e58dc1d72ef7579fba934aa0d073abf7b31210152236080d5e5a3fb3f  ankou-1.0.1-linux-x86_64.deb
```

L'empreinte calculée sur le fichier téléchargé est identique. Le nom de l'actif correspond
aussi au motif du manifeste (`ankou-{version}-{os}-{arch}.{ext}`).

**Première tentative**, `dpkg -i` seul, telle que le launcher la lance :

```
Unpacking ankou (1.0.1-1) ...
dpkg: dependency problems prevent configuration of ankou:
 ankou depends on nftables; however:
  Package nftables is not installed.
dpkg: error processing package ankou (--install):
 dependency problems - leaving unconfigured
```

**Constat important** : après cet échec, `dpkg-query -W -f='${Version}' ankou` répond
toujours `1.0.1-1`. Un paquet déballé mais non configuré **est** visible par la détection :
le launcher affichera donc « installée » alors que la configuration a échoué. C'est un
comportement à connaître, pas un bug de la détection : dpkg considère le paquet présent.

**Configuration complétée** par `apt-get install -y -f`, qui installe la dépendance et
configure le paquet :

```
Setting up nftables (1.1.6-1) ...
Setting up ankou (1.0.1-1) ...
Created symlink '/etc/systemd/system/multi-user.target.wants/ankoud.service'
  → '/usr/lib/systemd/system/ankoud.service'.
```

État final et version lue par la commande exacte du launcher :

```
Status: install ok installed
Version: 1.0.1-1
$ dpkg-query -W -f='${Version}' ankou
1.0.1-1
```

`1.0.1-1` analysé par `version_from_deb` donne `1.0.1` : la révision de paquet est bien
retirée. Ce cas est ajouté aux tests.

**Désinstallation** : `dpkg --purge ankou` supprime le paquet, le binaire et le service ;
`dpkg-query` ne trouve plus rien, et `systemctl is-enabled ankoud` répond `not-found`.
La dépendance `nftables`, installée uniquement pour ce test, a été retirée ensuite.

## Windows — détection vérifiée, installation non exécutée

L'entrée réelle, lue dans les clés de désinstallation :

```
DisplayName     : aNkou
DisplayVersion  : 1.0.0
Publisher       : LPDNSoft
```

C'est exactement ce que `installed_version` consomme : `DisplayName` contient le nom du
manifeste, `DisplayVersion` donne `1.0.0`. La capture d'écran du launcher le confirme — la
ligne aNkou affiche 1.0.0 face à la release 1.0.1.

**L'installation n'a pas été lancée** pour deux raisons, énoncées franchement :

1. ce shell n'est pas élevé, et une installation MSI machine entière exige une approbation
   UAC que je ne peux pas donner à votre place ;
2. installer aNkou 1.0.1 sur votre poste est une modification de votre système, pas un test.

**Ce qu'il faut lancer** pour terminer cette ligne :

```bat
gh release download v1.0.1 -R LordPhenixDeNetra/ankou -p "ankou-1.0.1-windows-x86_64.msi" -D "%TEMP%\ankou"
msiexec /i "%TEMP%\ankou\ankou-1.0.1-windows-x86_64.msi"
```

Puis, dans le launcher : la ligne doit passer de « MISE À JOUR DISPONIBLE 1.0.0 → 1.0.1 » à
« À JOUR 1.0.1 ». Vérification en ligne de commande :

```bat
reg query "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall" /s /f "aNkou" /d
```

## macOS — non exécuté

Aucune machine macOS n'était disponible. Ce qu'il faut lancer sur un Mac :

```sh
gh release download v1.0.1 -R LordPhenixDeNetra/ankou -p "ankou-1.0.1-macos-arm64.pkg" -D /tmp/ankou
installer -pkg /tmp/ankou/ankou-1.0.1-macos-arm64.pkg -target /
pkgutil --pkg-info fr.lpdnsoft.ankou
```

Attendu : `version: 1.0.1`, et `fr.lpdnsoft.ankou` dans la sortie — l'identifiant du
manifeste, que la détection interroge.

## Ce que cette vérification a changé

- Un test de plus : `version_from_deb("1.0.1-1")` vaut `1.0.1`, valeur réellement observée.
- Une limite documentée : un paquet déballé mais non configuré est compté comme installé.
- La confirmation que les noms d'actifs, le format de `SHA256SUMS` et les noms de paquets du
  manifeste correspondent à ce que la release publie réellement.
