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

Le core utilise `zip` (Stored et Deflate), `unrar` pour lire les RAR et `tempfile` pour préparer les exports. L'application utilise egui via eframe
avec le moteur OpenGL (`glow`), et `rfd` pour les sélecteurs natifs.
Sous Linux, un portail de bureau XDG est nécessaire pour les sélecteurs. `Cargo.lock` est conservé dans Git.
Un environnement graphique de bureau et un pilote OpenGL sont nécessaires.

## Interface egui

`cargo run -p cpm-app` lance l'éditeur de packs :

1. Ouvrir **File → Importer**, puis choisir **Fichier (.dynx)**, **Dossier
   de pack**, **Pack (.dnxpack)** ou **ZIP (.zip)** ou **RAR (.rar)**. Le menu est désactivé si
   des modifications non enregistrées risquent d'être abandonnées. Annuler
   le sélecteur conserve le document courant.
2. Les configurations sont classées à gauche par type (véhicules, moteurs,
   sons, roues, etc.). Les types d'addons inconnus restent dans **Autres**.
   Seul le nom du fichier apparaît ; le chemin complet est disponible au survol.
3. Les propriétés générales restent au centre. Les familles de sections,
   telles que **Point**, **Gear**, **Shape**, **Wheel** et **Seat**, se déplient
   en un groupe puis en sections individuelles. Les noms et les identités du
   fichier restent inchangés, y compris pour les doublons.
4. Cliquer sur une section à gauche ouvre son groupe et sa section au centre,
   puis fait défiler jusqu'à elle. **Racine** rejoint les propriétés générales.
5. Modifier les valeurs, puis indiquer un chemin libre et utiliser
   **Enregistrer une copie** pour la configuration sélectionnée, ou **File → Exporter** pour le pack entier.

La navigation ne perd pas les modifications des autres configurations.
**Annuler les modifications du fichier** restaure uniquement le fichier actif
à son dernier état enregistré. L'ouverture d'un autre fichier ou pack est bloquée tant
qu'un document visité reste modifié. La fermeture demande de confirmer l'abandon
des modifications de l'ensemble des fichiers.

La sauvegarde crée une copie de la configuration active, pas un nouveau pack.
Elle refuse tout chemin existant ; pour une sauvegarde suivante, choisir un autre
nom. Une erreur d'écriture peut laisser une copie incomplète. Le dossier ou
l'archive source n'est pas réécrit par cette commande. Le chemin de sauvegarde
individuelle reste saisi manuellement. L'export du pack complet est décrit ci-dessous ;
la 3D reste à ajouter.

## Première API du core

- `Pack::open` indexe récursivement les configurations `.dynx` d'un dossier
  ou d'une archive ZIP/DNXPACK/RAR, avec `pack_info.dynx` obligatoire à la racine.
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
4. Ajouter la prévisualisation 3D et des contrôles métier supplémentaires.

## Nom et icônes de l'application

La crate reste nommée `cpm-app` ; son exécutable est désormais `cpm`
(`cpm.exe` sur Windows). La commande de lancement reste :

```sh
cargo run -p cpm-app
```

Pour une version distribuable sous Windows :

```sh
cargo build -p cpm-app --release
```

Le résultat est `target/release/cpm.exe`. Les anciens fichiers `cpm-app.exe`
ne sont pas supprimés automatiquement par Cargo.

Le logo existant de CPM est utilisé pour l'icône de fenêtre. Une version ICO
multirésolution est aussi incorporée à l'exécutable Windows, avec les
métadonnées ProductName, FileDescription et OriginalFilename, par `build.rs`
et `winresource`. Cette compilation nécessite les outils de ressources du
Windows SDK (normalement présents avec les outils C++ pour Rust MSVC).
Le cache d'icônes de l'Explorateur Windows peut retarder l'affichage du logo.

## Import RAR

**File → Importer → RAR (.rar)** lit directement les archives RAR sans lancer
WinRAR ou un autre programme. `unrar_sys` compile sa bibliothèque C++ incluse :
un compilateur C++ est donc nécessaire à la compilation (outils Visual Studio
C++ sur Windows). Les RAR chiffrés et multivolumes ne sont pas pris en charge.
`pack_info.dynx` doit être à la racine. Une configuration est limitée à 8 Mio ;
une ressource RAR à exporter est limitée à 512 Mio. CPM ne crée pas de RAR.

## Export du pack complet

Avec un pack ouvert, **File → Exporter** affiche une fenêtre dédiée :

- Le dossier de destination se choisit avec **Parcourir…** ; son chemin reste
  visible et modifiable. Choisir un dossier en dehors du pack source.
- Choisir **Dossier**, **.zip** ou **.dnxpack**. Le nom du fichier/dossier de
  sortie est distinct du nom interne `PackName`. L'extension est ajoutée pour
  les archives.
- Cocher **Remplacer la sortie si elle existe déjà** uniquement pour remplacer
  exactement le chemin affiché. Avec le dossier parent, le même nom de sortie
  et le même format, cela permet de remplacer le pack source. Changer le
  format ou le nom crée une autre sortie et conserve l'original.
- Modifier le nom interne, la version et la description. `PackName` et
  `PackVersion` sont mis à jour dans `pack_info.dynx`. La description est
  conservée dans un champ `Description` sur une ligne ; ce champ complémentaire
  n'est pas documenté comme métadonnée standard par DynamX et peut être ignoré
  par le mod.

L'export inclut les configurations actives et mises en cache, les modèles,
textures, sons et autres ressources, ainsi que les dossiers vides. Les fichiers
non modifiés sont conservés ; les documents ne sont pas reconstruits à partir
d'un schéma limité. Les liens symboliques ne sont pas exportés.

Lors d'un changement de `PackName`, les références internes connues
`AttachedWheel`, `DefaultEngine` et `DefaultSounds` commençant par l'ancien nom
sont actualisées. Les références dans d'autres packs et celles propres aux
addons restent à vérifier. Les configurations doivent être lisibles pour
mettre ces références à jour.

Le traitement s'exécute en arrière-plan ; l'édition et la fermeture sont
bloquées pendant l'export. La sortie est préparée dans un dossier temporaire
sur le même volume avant installation. Une sortie existante est déplacée en
sauvegarde, puis restaurée si l'installation échoue. Si la restauration échoue,
le message indique le chemin de récupération, qui est conservé. Cette procédure
ne constitue pas une garantie contre une coupure électrique ou une modification
concurrente du disque par une autre application.

Après un export réussi, CPM ouvre le pack exporté, dont les modifications sont
alors enregistrées. L'export du pack entier n'est pas proposé pour un fichier
`.dynx` isolé : ouvrir le dossier ou l'archive du pack pour l'utiliser.
