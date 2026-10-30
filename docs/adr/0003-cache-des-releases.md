# ADR 0003 — Cache des releases : requêtes conditionnelles et repli hors ligne

- Statut : accepté
- Date : 2026-10-30
- Complète : ADR 0002

## Contexte

L'ADR 0002 fixait deux limites assumées : le quota de **soixante requêtes par heure** sans
authentification, et une liste vide quand le réseau manque. À deux applications cela tient ;
à dix, le quota tombe en quelques rafraîchissements, et hors ligne le launcher n'affiche
plus rien alors qu'il a déjà vu les données.

## Décision

1. **Requête conditionnelle.** L'empreinte de la dernière réponse part dans `If-None-Match`.
   Un **304** n'est pas une erreur : il signifie que le cache reste valable, et GitHub ne le
   compte pas dans le quota. `classify` le traite donc comme une issue normale, à côté du 200.

2. **Un fichier par application**, `<config>/lpdnsoft-launcher/cache/<id>.json`, contenant
   l'empreinte et la release. Écriture atomique (`lpdnsoft-atomic`), donc jamais de fichier
   à moitié réécrit.

3. **Un cache illisible est ignoré, pas fatal.** Le pire qu'il puisse arriver est une requête
   complète ; une exception ferait perdre la vérification entière.

4. **Repli hors ligne.** Si la requête échoue (réseau, quota), la dernière réponse connue
   sert de donnée, et l'interface l'annonce : « VÉRIFIÉ · DONNÉES EN CACHE », dans la couleur
   d'alerte. Présenter des données anciennes comme fraîches serait exactement le défaut
   corrigé en v0.2.

5. **Le cache n'est pas une source de vérité.** Aucune expiration : c'est le 304 qui décide,
   pas une durée arbitraire. Une release retirée côté GitHub reste donc visible jusqu'à la
   prochaine réponse 200 — assumé, et corrigeable plus tard par une durée de vie si le besoin
   apparaît.

## Conséquences

- **Le quota devient un non-sujet** tant que les réponses ne changent pas : chaque passe
  coûte au plus une requête par application *modifiée*.
- **Une première passe hors ligne reste muette** : sans cache, il n'y a rien à montrer. Le
  message d'échec le dit, avec son motif.
- **Le dossier de configuration dépend de la plateforme** (`directories`), donc du dépôt
  `lpdnsoft-settings` : c'est ce qui a fait entrer `option-ext` (MPL-2.0) dans l'arbre, d'où
  l'exception de licence, identique à celle d'`ankou`.
- **Pas de test du 304 réel** : le comportement réseau ne se teste pas sans serveur. Ce qui
  est testé, c'est la classification du statut, le repli sur cache, et le drapeau « données
  en cache » ; le reste a été vérifié en lançant l'application deux fois.
