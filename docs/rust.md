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
4. Étendre la prévisualisation 3D aux normal maps, animations et autres formats.

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

Les paramètres en erreur affichent leur nom et leur valeur en rouge dans la vue Propriétés, avec le diagnostic au survol. Les avertissements sont jaunes. Un paramètre obligatoire absent apparaît dans sa section avec un bouton **+ Ajouter** ; les titres des sections et groupes concernés sont également colorés lorsqu’ils sont repliés. La couleur disparaît après correction. Dans le catalogue Propriétés, les définitions invalides (nom, alias dupliqué, choix ou valeur par défaut incompatible) sont aussi marquées en rouge dans la liste, avec le motif à droite.


## Aperçu 3D des modèles OBJ

Le bouton **Aperçu 3D**, placé à côté du nom du fichier de configuration, ouvre une fenêtre redimensionnable. Il est activé lorsque le champ racine `Model` désigne un OBJ présent dans le pack. S’il est désactivé, le survol explique le problème (modèle absent, format différent ou moteur OpenGL indisponible). Le chemin du modèle est visible dans la fenêtre ; si la configuration change pendant que la fenêtre reste ouverte, elle s’actualise après une pause de saisie si l’actualisation automatique est activée.

- Glisser avec le bouton gauche : tourner autour du modèle.
- Molette au-dessus de l’aperçu : zoomer.
- Glisser avec le bouton droit : déplacer la vue.
- **Recentrer**, **Face**, **Côté**, **Dessus** : retrouver une vue pratique.

La géométrie est chargée sur un thread séparé depuis un dossier, un ZIP, un DNXPACK ou un RAR, sans extraction sur disque. Les fichiers `.dynx` ouverts seuls peuvent retrouver les ressources dans leur dossier de pack (premier ancêtre contenant `pack_info.dynx`) ou, à défaut, leur propre dossier. Les chemins relatifs à la racine du pack et au fichier de configuration sont reconnus, ainsi que les ressources de la forme `namespace:models/car.obj` dans `assets/namespace/`. Pour les chemins sans namespace, CPM recherche aussi dans `models/`, `assets/dynamxmod/`, `assets/dynamxmod/models/` et les mêmes dossiers sous `assets/minecraft/`. Les chemins sortant du pack sont refusés.

L’aperçu montre la géométrie complète, centrée automatiquement, avec éclairage simple et tampon de profondeur OpenGL. Il applique les couleurs diffuses `Kd`, l’opacité `d` et les textures `map_Kd` des matériaux MTL, en utilisant les coordonnées UV de l’OBJ. Il assemble les roues externes selon les règles ci-dessous ; les autres pièces ajoutées et animations ne sont pas appliquées. Les variantes diffuses sont décrites dans le guide des outils d’édition. Le chargeur accepte les faces triangulées et les polygones convertibles en éventail (notamment les quads), pas les polygones concaves arbitraires. Les fichiers illisibles ou invalides sont signalés dans la fenêtre. Limites : 32 Mio pour l’OBJ et un million de triangles. Le rendu nécessite OpenGL 3.1 ou OpenGL ES 3.


### Textures et pièces de l’OBJ

Les fichiers déclarés par `mtllib` sont lus relativement à l’OBJ ; les images `map_Kd` sont lues relativement au MTL. Les chemins `../` sont acceptés tant qu’ils restent dans le pack. Les ressources `namespace:chemin` sont également reconnues sous `assets/namespace/`. Aucune ressource située en dehors du pack n’est lue. Les images PNG, JPEG, TGA et BMP sont prises en charge. Plusieurs matériaux qui utilisent la même image partagent une seule texture chargée.

La case **Afficher les textures** permet de comparer le modèle texturé avec ses seules couleurs de matériaux. Les fichiers MTL ou images manquants, les UV absents et les images trop grandes sont signalés dans **Avertissements** ; la géométrie reste disponible avec une couleur de remplacement. Les options `map_Kd` commençant par `-` ne sont pas prises en charge et produisent un avertissement. Les normal maps, textures spéculaires et cartes d’opacité séparées ne sont pas appliquées. Le sélecteur de variantes prend en charge `MaterialVariants` et l’ancien champ `Textures`. La transparence de la texture diffuse et l’opacité du matériau sont rendues par tri des lots de faces ; les surfaces transparentes qui s’intersectent peuvent présenter des approximations.

La section **Pièces** reprend les noms des objets `o` et groupes `g` de l’OBJ. Chaque case permet d’afficher ou masquer une pièce ; **Isoler** affiche seulement cette pièce, **Tout afficher** rétablit le modèle complet et **Tout masquer** vide la vue. Le survol du nom affiche son nombre de triangles et ses matériaux. Un objet utilisant plusieurs matériaux reste une seule pièce avec plusieurs lots de rendu. Les noms identiques sont regroupés ; sans objet ou groupe nommé, la géométrie apparaît sous `unnamed_object`. Cette liste ne modifie pas les fichiers du pack.

Les textures sont chargées sur le même thread de travail que la géométrie. Limites : 4 Mio par MTL et 16 Mio de MTL par aperçu, 32 Mio par image compressée, dimensions maximales de 4096 × 4096 et 256 Mio de pixels décodés au total. Les textures GPU sont libérées à la fermeture ou au remplacement de l’aperçu.


## Créer et gérer les configurations

**File → Créer** ouvre un assistant guidé permettant de choisir un nouveau pack ou une configuration : véhicule, moteur, roue, sons, remorque, bateau, hélicoptère, bloc, prop, item, armure ou fichier personnalisé pour un addon.

Pour un pack, renseigner le nom, la version, `DcFileVersion` et la description. CPM prépare un brouillon avec son `pack_info.dynx` ; **File → Exporter** permet ensuite de choisir la destination et le format (dossier, ZIP ou DNXPACK). Le brouillon n’est pas ajouté aux packs récents avant son premier export. **File → Abandonner le brouillon** permet de le fermer sans créer de sortie.

Pour une configuration, saisir son nom et, dans un pack, un sous-dossier facultatif. Le préfixe et l’extension sont ajoutés automatiquement pour les types connus ; le type personnalisé conserve le nom de base saisi. Le formulaire utilise les paramètres racine obligatoires du catalogue actif, avec leurs valeurs par défaut disponibles. Une case permet aussi d’inclure les paramètres facultatifs ayant une valeur par défaut. Les champs obligatoires encore incomplets peuvent être complétés après création dans l’éditeur ; ils restent contrôlés avant export. Une configuration créée sans pack est un brouillon de fichier individuel, enregistré par **File → Exporter**.

Le bouton crayon **✎**, à côté de **Configurations**, ouvre la gestion des fichiers `.dynx` du pack courant. Il permet de créer une configuration, d’importer un `.dynx` existant à la racine ou de supprimer une configuration après confirmation. L’import conserve le fichier source intact. Les noms en double (y compris les différences de casse), chemins sortant du pack et noms de fichiers non portables sont refusés. `pack_info.dynx` est protégé contre la suppression.

Les changements de liste sont gardés en mémoire, y compris pour les archives RAR. **File → Exporter** les applique à la sortie en conservant les ressources du pack. Ils ne modifient pas directement le dossier ou l’archive source. **File → Annuler les ajouts et suppressions** rétablit la liste initiale, enlève les nouveaux fichiers et conserve les modifications des configurations existantes encore ouvertes. La suppression abandonne les modifications en mémoire du fichier supprimé ; les références à ce fichier dans les autres configurations ne sont pas réécrites automatiquement.


## Descriptions et saisie des paramètres

Dans **Propriétés**, chaque définition possède une **Description (affichée au survol)**, enregistrée par le bouton de sauvegarde du catalogue. Elle apparaît au survol du paramètre dans l’éditeur et dans le menu d’ajout. Les addons peuvent fournir le même champ JSON facultatif `description`. Les anciens fichiers restent compatibles. Les descriptions DynamX fournies sont ajoutées en mémoire aux anciennes définitions de `dynamx.json` sans ce champ ; les textes personnalisés, même explicitement vides, sont conservés.

Les champs de type entier, décimal, vecteur ou quaternion refusent la saisie et le collage de texte incompatible. Les signes, nombres décimaux et exposants restent possibles ; la virgule décimale est convertie en point. Les états intermédiaires nécessaires à la frappe (par exemple `-` ou `1e-`) restent autorisés mais doivent être complétés avant export. `MaxVehicleSpeed` conserve sa valeur spéciale `infinite`. Les paramètres texte restent libres. Les valeurs invalides déjà importées sont conservées et signalées pour correction.

### Roues dans l’aperçu

Pour un véhicule, l’aperçu recherche les configurations `AttachedWheel` du pack courant et ajoute leurs OBJ texturés aux positions des sections `Wheel…`. Il tient compte de `ScaleModifier` du véhicule et de la roue, de `IsRight` et de `SuspensionAxis`. Les positions utilisent le repère de configuration DynamX (`x y z` devient `x z -y`). Le champ `Scale` d’une section de roue concerne sa forme physique et ne redimensionne pas son OBJ.

Les pièces ajoutées apparaissent sous le nom de leur section de roue dans la liste des pièces. Les modifications de configuration en mémoire déclenchent une actualisation après une pause de saisie (case Actualisation automatique). Les roues d’un autre pack, modèles manquants, références ambiguës et articulations non simulées produisent un avertissement sans empêcher l’affichage du véhicule. Une section avec `Rim` désigne une roue intégrée au modèle et ne reçoit pas de doublon externe. Il s’agit d’un assemblage statique, sans simulation de suspension ni animation. Limites : 64 roues, un million de triangles et 256 Mio de textures pour la scène assemblée.


### Volant et boîtes de collision

L’aperçu place la pièce OBJ désignée par `ObjectName` (par défaut `SteeringWheel`, ancien alias `PartName`) dans les sections `SteeringWheel…`. Une `Position` explicite applique la translation configurée et `Rotation` applique la rotation de base ; `BaseRotation` et `BaseRotationQuat` sont aussi reconnus. Sans position explicite, le pivot est lu au centre de la pièce et sa pose de repos reste celle du modèle. Le volant n’est pas dupliqué, ses matériaux et son bouton de visibilité sont conservés. Une pièce absente ou une transformation invalide est signalée dans les avertissements. Aucun braquage animé n’est simulé.

**Afficher les collisions** active les douze arêtes vertes de chaque section racine `Shape…` de type `BOX` (type par défaut). Les lignes restent visibles à travers le modèle ; **Tout masquer** dans les pièces permet aussi d’examiner les boîtes seules. Le survol de la case affiche leurs noms. `Position` et `Scale` utilisent les conventions DynamX : position `x z -y`, demi-dimensions `x z y`, avec application de `ScaleModifier`. Les anciens noms `ShapePosition`, `Size`, `ShapeScale` et `BoxDim` sont acceptés. Le cadrage inclut les boîtes afin qu’elles restent visibles même hors de la carrosserie.

Cet affichage représente les boîtes déclarées dans la configuration, dans le repère du modèle ; il ne reconstruit pas les collisions complexes issues de l’OBJ ou des fichiers `.dc`, ni les volumes automatiques. Les sphères et cylindres ne sont pas remplacés par des boîtes : un avertissement indique qu’ils ne sont pas affichés. Le centre de gravité, les décalages de physique et les articulations ne sont pas simulés. Limite : 1024 boîtes par aperçu. L’actualisation automatique prend en compte les changements de configuration après une courte pause de saisie.

Les quaternions de configuration DynamX (`Rotation`, `SuspensionAxis`) sont lus dans l’ordre **w x y z** ; l’identité est `1 0 0 0`. Le catalogue fourni utilise cet ordre pour ses descriptions et valeurs par défaut.


## Nouveaux outils d’édition

Voir [le guide des outils](editor-tools.md) pour l’historique, la recherche, le diagnostic, l’arbre de modèles, les variantes et les poignées 3D.


## Assistant de création

Le nouveau parcours importe les modèles, matériaux et textures en mémoire, prépare les roues et les autres éléments en un seul lot et ne modifie le pack qu’après confirmation du récapitulatif. Voir [le guide de création pas à pas](creation-wizard.md). Les ressources préparées sont utilisables dans l’aperçu avant export. Annuler les ajouts et suppressions retire aussi ces ressources préparées.
