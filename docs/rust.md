# CPM en Rust

Le workspace contient `cpm-core`, une bibliothèque sans dépendance graphique,
et `cpm-app`, le point d'entrée de la future application. Le code Java dans
`src/` reste une référence pendant la migration.

## Démarrer

Avec Rust 1.95 ou supérieur et Cargo installés, depuis la racine :

```sh
cargo check --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p cpm-core --example edit_config
cargo run -p cpm-app
```

Le core utilise `zip` pour les archives (Stored et Deflate). L'application utilise egui via eframe
avec le moteur OpenGL (`glow`), et `rfd` pour les sélecteurs natifs.
Sous Linux, un portail de bureau XDG est nécessaire pour les sélecteurs. `Cargo.lock` est conservé dans Git.
Un environnement graphique de bureau et un pilote OpenGL sont nécessaires.

## Interface egui

`cargo run -p cpm-app` lance l'éditeur de packs :

1. Cliquer sur **Configuration (.dynx)…** pour une configuration isolée,
   **Pack ZIP / DNXPACK…** pour une archive `.zip` ou `.dnxpack`, ou **Dossier
   de pack…** pour un dossier contenant `pack_info.dynx`. Les archives se
   sélectionnent dans une fenêtre de fichiers ; le sélecteur de dossier sert
   uniquement aux packs non compressés. Le filtre **Tous les fichiers** reste
   disponible dans les fenêtres de fichiers. Ces boutons affichent le sélecteur
   du système. Annuler la sélection conserve le document courant.
   La saisie manuelle d'un chemin reste possible avec le bouton **Ouvrir**.
2. Choisir une configuration dans la liste de gauche : seul le nom du fichier
   est affiché, avec son chemin relatif au survol pour distinguer les homonymes.
3. Les propriétés de racine et toutes les sections sont regroupées au centre.
   Déplier les sections souhaitées avant le bloc **Texte du fichier après
   modification**. Cliquer sur une section à gauche la déplie et fait défiler
   la vue jusqu'à elle ; **Racine** ramène aux propriétés générales.
4. Modifier ses valeurs. Les documents visités restent en mémoire et les
   fichiers modifiés sont marqués d'un point dans la liste.
5. Indiquer un chemin libre et cliquer sur **Enregistrer une copie** pour
   sauvegarder la configuration sélectionnée.

La navigation ne perd pas les modifications des autres configurations.
**Annuler les modifications du fichier** restaure uniquement le fichier actif
à son dernier état enregistré. L'ouverture d'un autre fichier ou pack est bloquée tant
qu'un document visité reste modifié. La fermeture demande de confirmer l'abandon
des modifications de l'ensemble des fichiers.

La sauvegarde crée une copie de la configuration active, pas un nouveau pack.
Elle refuse tout chemin existant ; pour une sauvegarde suivante, choisir un autre
nom. Une erreur d'écriture peut laisser une copie incomplète. Le dossier ou
l'archive source n'est pas réécrit. Le chemin de sauvegarde reste saisi
manuellement. L'export du pack entier et la 3D restent à ajouter.

## Première API du core

- `Pack::open` indexe récursivement les configurations `.dynx` d'un dossier
  ou d'une archive, avec `pack_info.dynx` obligatoire à la racine.
- `Pack::read_config` charge uniquement le document demandé ; un fichier
  malformé ne bloque pas la navigation vers les autres.
- Les archives sont lues sans extraction ; les chemins non sûrs, les doublons
  de configurations et les liens de configurations en archive sont refusés.
  Les liens symboliques des dossiers sont ignorés, et une configuration est
  limitée à 8 Mio décompressés. Les ressources ne sont pas chargées en mémoire.
- `Document::parse` et `read_from` lisent une configuration UTF-8.
- `properties()` expose les clés, valeurs, sections et numéros de ligne.
- `sections()` expose les noms et parents des sections ; leurs indices sont
  locaux au document. Les sections et clés dupliquées restent distinctes.
- `set_value(index, value)` modifie une propriété existante par son indice.
  Les indices restent stables après modification. Les valeurs multilignes
  sont refusées ; les espaces externes de la nouvelle valeur sont retirés.
- `source()` et `write_to` restituent le texte, avec ses commentaires, son
  ordre, ses champs inconnus, son BOM éventuel et ses fins de ligne.
- `PackInfo::from_document` vérifie la présence de `PackName` et
  `DcFileVersion` à la racine. Les champs vides ou dupliqués sont signalés.
  Cette vue ne remplace pas le document à sauvegarder.

Le lecteur distingue exactement les clés et découpe les propriétés au premier
deux-points, pour conserver notamment les chemins `modid:ressource`.
Il accepte les commentaires de ligne `//`, les blocs de commentaire `/* … */`
sur des lignes dédiées et les sections dont l'accolade ouvrante suit le nom.
Les erreurs syntaxiques indiquent la ligne concernée.

## Référence et limites

Implémentation basée sur la documentation fournie :

- [Syntaxe des configurations](https://wiki.dynamx.fr/fr/dynamx/DynamXPackConfigsSyntax/)
- [Structure et métadonnées des packs](https://wiki.dynamx.fr/fr/dynamx/DynamXPackBase/)

Les tests utilisent des configurations synthétiques ; la compatibilité avec
des packs réels et toutes les extensions d'addons reste à vérifier. Le lecteur
ne valide pas encore la contrainte des frontières de sections dans les
commentaires multilignes. Il ne traite pas les commentaires en fin de propriété.

Cette première version n'interprète pas encore les champs des véhicules et ne gère pas l'ajout ou la
suppression de propriétés. La validation des versions, des références et des
types de valeurs reste à implémenter. `write_to` écrit dans un flux : l'appelant
doit gérer les sauvegardes atomiques et les copies de secours sur disque.

## Prochaines étapes

1. Ajouter des packs réels de référence et vérifier leurs cycles de sauvegarde.
2. Ajouter la recherche dans les configurations.
3. Ajouter les vues métier (véhicules, roues, sièges) et leurs validations.
4. Ajouter un export du pack complet et la prévisualisation 3D.
