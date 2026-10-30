# Plan — installer et mettre à jour

- **Date :** 2026-10-30
- **Spécification :** `docs/superpowers/specs/2026-10-30-installation.md`
- **Décision :** ADR 0005
- **Règle :** chaque étape se termine par `cargo xtask ci` en exit 0.

---

## Étape 1 — Le plan d'installation, pur

**Changement.** `crates/install` : `plan(paquet, système)` rend `Plan { programme, args,
elevated, summary }` ; refuse une extension incompatible (`Unsupported`) et un chemin
dangereux (`Refused`).

**Tests.** Un plan par système (MSI, deb, rpm, pkg) ; extension incompatible ; cinq chemins
piégés ; le chemin apparaît tel quel dans les arguments.

**Fini quand.** les quatre plans sont assertés et les refus aussi.

---

## Étape 2 — L'enveloppe d'élévation

**Changement.** `elevated(plan, système)` produit la commande finale : `pkexec`, `powershell
… -Verb RunAs …`, `osascript … with administrator privileges`.

**Tests.** Programme et arguments de l'enveloppe ; la commande de base reste intacte à
l'intérieur ; un plan déjà élevé n'est pas enveloppé deux fois.

**Fini quand.** les trois enveloppes sont assertées.

---

## Étape 3 — L'exécution injectable

**Changement.** Trait `Executor`, `SystemExecutor` (lance le plan et rend le code de sortie),
`FakeExecutor` pour les tests.

**Tests.** Le faux exécutant reçoit exactement le plan ; un code non nul devient `Failed` avec
le code ; une commande absente devient un échec explicite.

**Fini quand.** rien n'est lancé pendant les tests, et le contrat est prouvé.

---

## Étape 4 — Brancher sur l'interface

**Changement.** `Request::Install(id)` : télécharger (ADR 0004), puis planifier, puis exécuter
sur un fil, en affichant « téléchargement », « vérification », « installation ». Les boutons
Install/Mettre à jour cessent d'être désactivés.

**Prérequis.** une vraie release à installer, et l'accord sur l'expérience d'élévation.

**Fini quand.** un clic mène de la ligne à l'installation, avec un état visible à chaque étape.

---

## Étape 5 — Vérification manuelle sur les trois systèmes

**Changement.** aucun code. Installer aNKou 1.0.0, puis 1.0.1, sur Windows, Debian/Fedora et
macOS. Vérifier : l'invite apparaît, l'installation aboutit, le launcher affiche la nouvelle
version, la désinstallation fonctionne.

**Fini quand.** les trois installations sont consignées avec leur sortie.

---

## Récapitulatif

| Étape | Nature | Tests |
|---|---|---|
| 1 | plan pur | 8 |
| 2 | élévation | 3 |
| 3 | exécution | 3 |
| 4 | interface | à définir |
| 5 | manuel | — |

Les étapes 1 à 3 sont livrées ici ; 4 et 5 demandent une release réelle et votre accord sur
l'expérience d'élévation.
