Première version publique de CPM réécrite en Rust avec egui, pour Windows 10/11 64 bits (Intel/AMD).

## Télécharger

- **cpm-setup-windows-x86_64.exe** : installateur recommandé. Installe CPM pour le compte Windows courant, sans droits administrateur, avec raccourci dans le menu Démarrer et option de raccourci sur le bureau.
- **cpm-windows-x86_64.exe** : version portable. Placer l’exécutable dans un dossier accessible en écriture puis le lancer directement.
- **SHA256SUMS.txt** : empreintes SHA-256 des deux exécutables.

Java, Maven, Rust et Cargo ne sont pas nécessaires pour utiliser ces exécutables. Les exécutables ne sont pas signés numériquement.

## Fonctionnalités

- Ouverture de packs en dossier, ZIP, DNXPACK ou RAR ; configurations `.dynx` individuelles et cinq packs récents.
- Navigation par catégories, paramètres et sections ; ajout/suppression de paramètres et diagnostics visibles.
- Catalogue de propriétés personnalisable et définitions JSON pour les addons.
- Création de packs/configurations ; import et suppression de fichiers via le crayon du panneau Configurations.
- Export complet en dossier, ZIP ou DNXPACK, avec modification des métadonnées du pack.
- Aperçu 3D OBJ : rotation, zoom, matériaux MTL, textures diffuses, masquage et isolation des pièces nommées.
- Liens wiki/bugs et vérification des nouvelles versions Rust.

## Données et limites

Les ajouts, suppressions et modifications du pack restent en mémoire jusqu’à l’export. Le catalogue est enregistré dans `properties/`, à côté de CPM, et l’historique dans `recent-packs.json`. Ces fichiers personnels sont conservés lors des mises à jour.

L’aperçu ne reproduit pas les animations, transformations ni variantes de textures DynamX. La validation couvre les paramètres du catalogue actif, pas l’ensemble des règles de tous les addons. L’export RAR n’est pas proposé ; les packs RAR peuvent être exportés en dossier, ZIP ou DNXPACK.

Rapports de bugs : https://github.com/Florent-Marc/cpm/issues

Cette version corrige la liaison Windows de la bibliothèque RAR détectée lors du premier build public 0.2.0.
