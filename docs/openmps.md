# Publier un pack avec OpenMPS

OpenMPS est une option de publication. **File → Exporter** continue à produire un dossier, un ZIP ou un DNXPACK sans compte, sans serveur et sans connexion OpenMPS.

## Une première fois côté serveur

Le serveur doit disposer de l’API d’administration v1 fournie dans le [patch compagnon](../integrations/openmps/README.md). L’OpenMPS d’origine ne fournit pas cette API. Définir son URL publique, puis créer un jeton sur le serveur :

```sh
php tools/api_token.php create CPM
```

Le jeton d’administration et la clé MPS du jeu ont des rôles différents. Le premier autorise CPM à publier ; la seconde figure dans le pack distribué aux joueurs, suivant le protocole MPS. Ne jamais mettre le jeton d’administration dans un pack.

## Dans CPM

1. Ouvrir le pack puis **File → Publier sur OpenMPS…**. **Outils → OpenMPS…** permet aussi de consulter le serveur sans pack ouvert.
2. Renseigner l’URL HTTPS et le jeton, puis **Tester la connexion / actualiser**. HTTP est accepté uniquement en local pour les tests. Un profil enregistre son nom et son URL dans `openmps-servers.json` à côté du programme ; le jeton reste en mémoire pour la session.
3. Choisir l’identifiant et la version, puis **Vérifier et préparer la publication**. Corriger les erreurs indiquées avant de continuer. La fenêtre présente les déplacements de ressources et les avertissements.
4. Indiquer la clé MPS du jeu autorisée par le serveur puis **Publier cette version sur OpenMPS**. L’envoi affiche une progression ; la préparation serveur se termine ensuite. Une version existante ne peut pas être remplacée par cette commande.
5. Dans la liste distante, **Télécharger le ZIP joueurs…** et choisir un nouveau fichier. CPM contrôle le SHA-256, les métadonnées et la présence des configurations avant de conserver le téléchargement. Un fichier existant n’est pas écrasé.

Après un délai dépassé ou une coupure, actualiser la liste : le serveur peut avoir terminé la publication malgré l’absence de réponse. La fermeture du programme attend la fin d’une opération en cours. Aucun pack n’est publié simplement en ouvrant cette fenêtre.

## Compatibilité et limites

Le diagnostic fige une copie temporaire à partir des modifications en mémoire. Il garde les `.dynx` hors de `assets/`, déplace les ressources vers `assets/<identifiant>/` si nécessaire et adapte les références `Model`, OBJ/MTL et les références internes connues lors du renommage du pack. Les ressources déjà sous `assets/<domaine>/` restent à leur place. Les fichiers de documentation et anciens fichiers MPS générés sont exclus de cette publication. Le pack de travail et ses exports habituels conservent leurs chemins.

Le catalogue ne connaît pas tous les addons. Des références particulières dans les addons ou JSON/glTF doivent être vérifiées manuellement ; les déplacements de JSON/glTF donnent un avertissement. Les options MTL complexes non prises en charge et les dépendances absentes bloquent la préparation plutôt que produire une référence cassée. La validation n’est pas un test du pack dans Minecraft.

Le ZIP joueurs contient les configurations et `MpsRepositories.json` ; les ressources restent servies par OpenMPS. CPM ne déploie pas de serveur et n’installe rien dans Minecraft. L’intégration cible le protocole MPS 1.3.0 et l’API `openmps-admin/1`.

## Vérifications de développement

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
OPENMPS_SOURCE=/chemin/vers/OpenMPS python3 integrations/openmps/test_roundtrip.py
```

Le dernier test utilise le serveur PHP réel avec des jetons et fichiers temporaires : préparation Rust, connexion, publication, répétition idempotente, liste, téléchargement vérifié et refus d’écraser un fichier local. Il nécessite PHP avec OpenSSL et ZipArchive et le patch compagnon appliqué.
