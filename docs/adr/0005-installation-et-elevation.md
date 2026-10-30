# ADR 0005 — Installation et élévation des privilèges

- Statut : accepté
- Date : 2026-10-30
- Complète : ADR 0002, ADR 0004
- Précédent : l'ADR 0004 réserve le numéro 0005 à cette décision

## Contexte

Le launcher sait vérifier et télécharger un paquet vérifié (ADR 0004). Il doit maintenant
l'installer. Or **tous** les installeurs de la suite exigent des droits administratifs :

| Système | Paquet | Commande | Portée |
|---|---|---|---|
| Windows | MSI | `msiexec /i <fichier> /qn /norestart` | machine entière, service |
| Linux | `.deb` | `dpkg -i <fichier>` | système |
| Linux | `.rpm` | `rpm -U <fichier>` | système |
| macOS | `.pkg` | `installer -pkg <fichier> -target /` | système, démon launchd |

C'est une contrainte de conception, pas un détail : un installeur qui pose un service ou un
démon ne peut pas s'installer dans le profil seul.

## Décision

1. **Élévation par installation, demandée par le système.** Pas de service privilégié
   résident, pas de lancement du launcher en administrateur : chaque installation déclenche
   l'invite du système d'exploitation (UAC, polkit, autorisation macOS), une seule fois.

   | Système | Mécanisme | Programme lancé |
   |---|---|---|
   | Windows | `Start-Process -Verb RunAs` | `powershell` |
   | Linux | polkit | `pkexec` |
   | macOS | `do shell script … with administrator privileges` | `osascript` |

2. **Aucun `unsafe`.** Le workspace l'interdit, et le contourner pour appeler
   `ShellExecuteW("runas")` serait un coût de sûreté pour un confort d'API. Les trois
   mécanismes ci-dessus sont des programmes ordinaires, donc lançables sans FFI.

3. **Le plan est pur, l'exécution est mince.** `plan(paquet, système)` rend la commande
   complète — programme, arguments, mention d'élévation — et ne lance rien. `elevated(plan)`
   l'enveloppe dans le mécanisme du système. Toute la logique est donc testable.

4. **Un chemin dangereux est refusé, jamais échappé.** Le chemin du paquet finit dans une
   ligne de commande PowerShell ou AppleScript : guillemets, point-virgule, `$`, backtick ou
   retour à la ligne y seraient une injection. Le plan refuse ces chemins (`Refused`) au lieu
   de tenter un échappement — le chemin est construit par le launcher, un refus est un bug
   visible, pas une perte.

5. **Rien ne s'installe sans redémarrage contrôlé.** `msiexec` reçoit `/norestart` : le
   launcher ne redémarre jamais la machine dans le dos de l'utilisateur.

6. **Pas de rollback en v1.** Un `.deb` ou un `.rpm` qui échoue à mi-parcours peut laisser un
   paquet à moitié configuré. On ne prétend pas l'annuler : on relaie le code de sortie et la
   sortie d'erreur de l'outil, l'utilisateur décide. Un vrai rollback suppose de garder la
   version précédente installée, ce qui est un autre métier.

## Conséquences

- **Une invite système par installation**, ce qui est le comportement attendu par tout
  utilisateur de MSI ou de `.deb`. Aucun mot de passe n'est demandé ni stocké par le launcher.
- **`pkexec` peut manquer** (poste sans polkit) : le plan le signale comme un échec explicite,
  avec la commande à lancer à la main. Ce n'est pas silencieux.
- **Windows : `powershell.exe` est requis.** Présent partout depuis Windows 7 ; son absence
  serait un système exotique, et l'échec sera explicite.
- **Le launcher ne peut pas installer en mode silencieux sans consentement** : l'invite est
  le consentement.
- **La mise à jour suit le même chemin** qu'une installation : c'est le même installeur, avec
  une version différente. Aucune commande distincte, donc aucun code distinct.
- **Ce qui reste à faire** : brancher le plan sur l'interface et sur le téléchargement,
  afficher la progression, et vérifier bout en bout sur les trois systèmes (étape 5 du plan).
