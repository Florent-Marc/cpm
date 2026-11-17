# CPM en Rust

Le workspace contient `cpm-core`, une bibliothèque sans dépendance graphique,
et `cpm-app`, l'application de bureau egui. Les anciens fichiers Java et Maven
ont été retirés de cette branche ; leur historique reste disponible dans Git.

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
5. Utiliser le petit **+** au-dessus des propriétés de la racine ou d'une
   section pour insérer un paramètre absent. Les obligatoires sont proposés
   en premier ; les paramètres déjà présents sont masqués. Le champ personnalisé
   accepte les paramètres d'addons.
6. Utiliser **File → Exporter** pour enregistrer le pack entier. Pour un fichier
   `.dynx` isolé, cette commande ouvre un sélecteur de fichier de sortie.
7. **Wiki**, à côté de **File**, ouvre directement la documentation DynamX
   dans le navigateur par défaut.

La navigation ne perd pas les modifications des autres configurations.
**File → Annuler les modifications du fichier** restaure uniquement le fichier actif
à son dernier état enregistré. L'ouverture d'un autre fichier ou pack est bloquée tant
qu'un document visité reste modifié. La fermeture demande de confirmer l'abandon
des modifications de l'ensemble des fichiers.

La ligne « Nouvelle copie » a été retirée de la fenêtre principale. L'export
individuel refuse d'écraser un fichier existant. L'export du pack entier et son
option de remplacement sont décrits ci-dessous.

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
- `remove_property(index)` retire seulement la ligne ciblée, sans supprimer
  les commentaires voisins ni la section. Les indices sont recalculés.
- `add_property(section, key, value)` insère une propriété à la racine ou avant
  la fermeture d'une section, sans déplacer les lignes existantes. Les doublons
  dans la même section et les noms invalides sont refusés. Les indices de
  propriétés peuvent changer après insertion : rafraîchir les valeurs affichées.
- `parameter_catalog` expose les paramètres connus et leurs valeurs proposées ;
  `configuration_issues` vérifie leurs champs obligatoires dans la bonne portée.
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

Un bouton **−** permet de supprimer chaque paramètre. Retirer un obligatoire
réactive le signalement et bloque l'export tant qu'il n'est pas rétabli.
L'ajout personnalisé s'ouvre désormais dans une fenêtre dédiée (nom et valeur),
qui reste ouverte pendant la saisie, hors du menu contextuel.

Le catalogue initial couvre les métadonnées, les champs de base des items,
blocs, armures et véhicules, les moteurs, roues, ainsi que des sections usuelles
(points, rapports, sièges, formes, sons et addons requis). Il n'est pas un
schéma exhaustif pour toutes les extensions DynamX. La validation des plages numériques et des références reste à implémenter. `write_to` écrit dans un flux : l'appelant
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

## Paramètres obligatoires et signalement

À l'ouverture d'un pack, les configurations sont vérifiées en arrière-plan.
Un nom de fichier et sa catégorie apparaissent en rouge si une configuration
est illisible ou si un paramètre obligatoire connu manque, est vide (sauf
texte descriptif autorisé vide) ou est dupliqué. Le survol du fichier affiche
son chemin et les raisons. Pour un document ouvert, le signalement reflète
immédiatement les modifications en mémoire.

Les mêmes règles bloquent l'export, avant installation de la sortie. Ajouter
un champ obligatoire vide ne suffit pas : il faut renseigner sa valeur.
`DcFileVersion` est proposé avec `12.5.0`, conformément à l'exemple du wiki.
Les propriétés facultatives absentes et les paramètres inconnus d'addons
ne sont pas signalés uniquement parce qu'ils sont inconnus.

Références du catalogue :

- [Métadonnées](https://wiki.dynamx.fr/fr/dynamx/DynamXPackBase/)
- [Véhicules et sections](https://wiki.dynamx.fr/fr/dynamx/vehicles/ModularVehicleInfo/)
- [Moteurs et sons](https://wiki.dynamx.fr/fr/dynamx/vehicles/EngineInfo/)
- [Roues](https://wiki.dynamx.fr/fr/dynamx/vehicles/WheelInfo/)
- [Items](https://wiki.dynamx.fr/fr/dynamx/DynamXItems/)
- [Blocs](https://wiki.dynamx.fr/fr/dynamx/DynamXPropsAndBlocks/)
- [Armures](https://wiki.dynamx.fr/fr/dynamx/DynamXArmors/)

## Version, bugs et mises à jour

La version provient de Cargo (`0.2.0` pour cette étape) et s'affiche en bas à
droite. **Report bug**, à droite de la barre de menus, ouvre le formulaire de
nouvelle issue du dépôt avec la version, le système et une trame préremplis.
Aucune issue n'est envoyée automatiquement : l'utilisateur complète le rapport
et le valide sur GitHub.

Au démarrage puis toutes les heures, CPM vérifie les releases publiques du
dépôt en arrière-plan, avec un délai réseau limité. **File → Vérifier les mises
à jour** relance la recherche. Un échec réseau n'empêche pas l'édition ; son
détail est consultable au survol de la version. Les drafts, préversions et
anciennes releases Java sont ignorés. Une release Rust utilise un tag
`rust-vMAJEUR.MINEUR.CORRECTIF`, cohérent avec la version Cargo.

Lorsqu'une version stable plus récente est disponible, un bouton apparaît dans
la barre supérieure. Sur Windows x86-64, si la release contient l'asset
`cpm-windows-x86_64.exe` avec sa somme SHA-256 GitHub, ce bouton télécharge,
vérifie et installe l'exécutable au redémarrage. Il est désactivé en présence
de modifications non enregistrées ou d'une saisie d'addon ouverte. Pendant le
téléchargement, l'édition et la fermeture sont bloquées.

Un script PowerShell attend la fermeture de CPM, conserve l'ancien exécutable,
remplace le fichier puis lance la nouvelle version. En cas d'échec, il tente de
restaurer l'ancien et conserve un journal et les fichiers de récupération dans
le dossier `.cpm-update-*` indiqué. Cette procédure Windows n'est pas validée
par les tests Linux ; vérifier l'installation réelle sur Windows avant une
distribution large. Le programme doit pouvoir écrire dans son dossier ; les
politiques Windows peuvent bloquer PowerShell ou l'exécutable téléchargé.

Sans binaire compatible ou sans somme de contrôle, et sur les autres systèmes,
le bouton ouvre la page GitHub de la release pour une installation manuelle.
Aucune mise à jour n'est déduite des simples commits de branche.

Le workflow `.github/workflows/rust-windows-release.yml` permet de vérifier et
compiler Windows manuellement (`workflow_dispatch`). Pousser un tag `rust-v*`
correspondant à la version Cargo déclenche aussi la publication d'une release
avec l'asset nommé comme attendu par le programme. Aucun tag ni release n'est
créé lors des modifications de code dans cette conversation.

## Branche Rust uniquement

Les sources Java, interfaces FXML, anciens JAR, fichiers Maven et traces de
crash de l'ancienne application ont été retirés de cette branche. Les ressources
utilisées par Rust (logo, icône Windows et script de mise à jour) restent dans
`crates/cpm-app/assets`. Les tests et leur fixture RAR restent dans le core.

Java et Maven ne sont nécessaires ni pour compiler ni pour lancer CPM Rust.
L'ancienne implémentation reste consultable dans l'historique Git et dans la
branche Java ; aucun historique n'a été réécrit lors de ce nettoyage.


## Catalogue de propriétés et addons

Le bouton **Propriétés**, à côté de File, ouvre l’éditeur des définitions utilisées par les boutons `+`, la validation et l’export. Choisir un fichier de catalogue, une règle de fichier/section puis un paramètre pour modifier son type, son caractère obligatoire, sa sévérité et sa valeur par défaut facultative. Les types disponibles comprennent texte, entier, décimal, booléen, vecteurs, quaternion, ressource, liste et choix prédéfinis. Un survol du nom d’un paramètre dans l’éditeur principal affiche sa définition.

**Enregistrer et appliquer** écrit les définitions et revalide les configurations ouvertes et en mémoire. **Fermer sans appliquer** abandonne le brouillon. Une erreur bloque l’export et colore la configuration en rouge ; un avertissement est affiché en jaune dans la configuration et ne bloque pas l’export. Ignorer désactive le diagnostic de ce paramètre. Un paramètre facultatif reste contrôlé s’il est présent. Une valeur par défaut sert de suggestion au bouton `+` ; elle ne modifie jamais automatiquement un pack.

Au premier lancement, CPM crée `properties/dynamx.json` à côté de l’exécutable avec le catalogue DynamX actuellement pris en charge. Le dossier doit être accessible en écriture. Tous les autres fichiers `.json` de ce dossier sont chargés comme catalogues d’addons : `dynamx.json` en premier, puis les addons par ordre alphabétique. À nom de paramètre et portée identiques, la dernière définition remplace entièrement la précédente. Les motifs de fichier portent sur le nom seul, sans chemin ; les motifs de section et de parent respectent la casse. L’absence de `section_pattern` cible la racine du fichier.

On peut créer un fichier addon dans la fenêtre ou y importer un JSON. Après ajout manuel dans le dossier, utiliser **Recharger le dossier** puis **Enregistrer et appliquer**, ou redémarrer CPM. Les fichiers invalides sont signalés et ignorés sans être effacés. Un catalogue DynamX invalide entraîne un repli en mémoire sur les définitions intégrées. Le remplacement de l’exécutable lors d’une mise à jour conserve ce dossier.

Exemple de fichier `properties/mon-addon.json` :

```json
{
  "schema_version": 1,
  "label": "Mon addon",
  "rules": [
    {
      "file_pattern": "vehicle_*.dynx",
      "parameters": [
        {
          "name": "AddonPower",
          "variable_type": "decimal",
          "required": true,
          "severity": "warning",
          "allow_empty": false,
          "default_value": "1.0"
        }
      ]
    }
  ]
}
```

Les valeurs de sévérité JSON sont `ignore`, `warning` et `error`. Les types sont `text`, `integer`, `decimal`, `boolean`, `vector2`, `vector3`, `quaternion`, `resource`, `enumeration` et `list`. Pour une énumération, ajouter `choices` (tableau de chaînes). `aliases` permet d’accepter d’autres noms pour un même paramètre. Pour une section, ajouter `section_pattern`, par exemple `Point_*`, et éventuellement `parent_pattern` pour limiter au parent direct. Les champs inconnus et versions de schéma non prises en charge sont refusés pour éviter les fautes silencieuses.

## Packs récents

**File → Ouvrir récent** affiche les cinq derniers packs distincts (dossiers, ZIP, DNXPACK ou RAR), du plus récent au plus ancien. Réouvrir le même pack le remonte sans doublon. Seules les ouvertures réussies sont mémorisées ; les fichiers `.dynx` individuels sont exclus. Le chemin complet apparaît au survol. La liste est conservée dans `recent-packs.json` à côté de l’exécutable. Comme pour Importer, il faut enregistrer ou annuler les modifications avant de changer de pack.
