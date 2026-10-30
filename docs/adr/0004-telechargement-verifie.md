# ADR 0004 — Téléchargement vérifié des paquets

- Statut : accepté
- Date : 2026-10-30
- Complète : ADR 0002, ADR 0003

## Contexte

Le launcher sait quelle version est publiée, mais il ne sait pas encore obtenir le fichier.
C'est pourtant la seule étape qui compte pour l'utilisateur, et la plus dangereuse : un
paquet téléchargé est un programme qu'on s'apprête à exécuter. Un téléchargement interrompu,
un miroir qui répond à moitié, ou une release altérée doivent échouer **avant** l'installation.

## Décision

1. **`SHA256SUMS` fait foi.** Chaque release publie ses empreintes (convention d'ANKOU,
   ADR 0011). Le téléchargement ne publie un fichier que si son empreinte correspond à celle
   du fichier portant le nom attendu. Sans cette correspondance, le fichier est supprimé.

2. **Taille vérifiée quand elle est connue.** L'API GitHub donne la taille de chaque asset ;
   une taille différente est un échec immédiat, avant même de calculer l'empreinte.

3. **Reprise par `Range`.** Le fichier arrive dans `<nom>.part`. Si une tentative précédente
   l'a laissé incomplet, la requête suivante demande `bytes=<n>-` ; un 206 ajoute la suite, un
   200 (serveur qui ignore `Range`) ou un 416 repartent de zéro.

4. **Publication atomique.** Le `.part` n'est renommé vers sa destination finale qu'une fois
   taille et empreinte confirmées. Un échec supprime le partiel : on ne laisse jamais un
   fichier à moitié bon traîner dans le cache.

5. **La décision est pure.** `plan(octets_présents, statut)` décide entre reprendre, repartir
   et refuser ; c'est testable sans serveur, et c'est la partie délicate.

6. **Testé contre un vrai serveur, local.** Les tests montent un serveur HTTP minimal dans un
   fil : 200, 206 honoré, 404. On vérifie ainsi le comportement réel sans réseau externe.

## Conséquences

- **Un paquet n'est jamais publié sans empreinte** : la fonction exige l'empreinte en
  paramètre, il n'existe pas de chemin « télécharger sans vérifier ».
- **Le cache des paquets n'existe pas encore** : le fichier est téléchargé dans le dossier
  de l'appelant. Où le poser, et quand le purger, sera tranché avec l'installation (ADR 0005).
- **GitHub redirige les assets** vers `objects.githubusercontent.com` ; le client suit les
  redirections, et `Range` traverse la redirection — vérifié par le comportement du client,
  pas par un test, faute de réseau dans la suite.
- **La reprise suppose un serveur coopératif.** Un miroir qui ne gère pas `Range` fait
  repartir de zéro : c'est plus lent, jamais faux.
