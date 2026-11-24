# CPM 0.3.0 — Windows et Linux

## Téléchargements

- **Windows x64** : `cpm-setup-windows-x86_64.exe` pour installer CPM ou `cpm-windows-x86_64.exe` en version portable.
- **Linux x86_64** : `cpm-linux-x86_64.tar.gz`, à extraire puis lancer avec `./cpm` dans le dossier CPM. Construction sur Ubuntu 22.04 ; nécessite une session graphique et OpenGL 3.1 / ES 3 pour l’aperçu 3D.
- `SHA256SUMS.txt` fournit les empreintes des quatre téléchargements.

## Définitions incluses

DynamX et BasicsAddon sont disponibles dès le premier lancement : types, descriptions, valeurs par défaut et paramètres proposés dans les sections compatibles. Les catalogues sont modifiables dans `properties/` ; les fichiers personnalisés existants ne sont pas remplacés. Les définitions BasicsAddon sont basées sur le dépôt DynamXInc/BasicsAddon, commit c4b10d795cc59925a30010c3427c849ec0320e61. Les mods Minecraft eux-mêmes ne sont pas inclus.

## Nouveautés et correctifs

- Interface harmonisée, accueil avec packs récents, menus simplifiés et accès direct à l’export.
- Champs alignés avec exemples et explications des erreurs ; actions d’ajout et de suppression explicites.
- Comparaison des changements avant export et édition groupée de configurations.
- Gestion de plusieurs serveurs OpenMPS, jetons séparés avec coffre système facultatif, onglets de publication et de récupération.
- Récupération des configurations et ressources originales depuis l’API OpenMPS avec le droit `recover` (mise à jour de l’API nécessaire).
- Aperçu 3D détachable pour un deuxième écran, options et pièces dans une colonne latérale.
- Déplacement et taille des collisions affichés en direct, sans rechargement du modèle pour chaque geste ; Ctrl+Z annule le geste complet.
- Sélection et rotation de caméra disponibles également en modes Déplacer et Taille.
- Prise en charge des modèles glTF statiques avec confirmation de la version bêta DynamX lors de leur ajout.

Les exports locaux restent disponibles sans OpenMPS. L’installation automatique des mises à jour reste réservée à Windows x64 ; les autres plateformes proposent la page de téléchargement.
