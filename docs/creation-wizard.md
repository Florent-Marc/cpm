# Créer un objet avec l’assistant

Ouvrir **File → Créer**, ou le crayon à côté de **Configurations**, puis **Créer une configuration**. L’assistant adapte ses étapes au type choisi : un moteur n’a pas besoin d’OBJ, une roue n’a pas besoin d’emplacements de roues, et les options de siège et de volant sont réservées aux véhicules.

## 1. Votre projet

Choisir véhicule, roue, bloc, prop, item, armure, etc., puis un nom de fichier court. Saisir par exemple `transit` : CPM ajoute le préfixe et l’extension, ici `vehicle_transit.dynx`. Les noms de fichiers générés utilisent des lettres, chiffres, tirets ou underscores, sans espace. Le nom affiché en jeu se règle séparément dans les propriétés.

Dans un pack ouvert, la configuration sera ajoutée à ce pack. Pour un objet avec modèle créé sans pack ouvert, l’assistant prépare aussi un nouveau pack : renseigner son nom interne, sa version, sa description et `DcFileVersion`. Il est également possible de demander un nouveau pack depuis un pack ouvert, après avoir enregistré ou annulé les changements en cours.

## 2. Modèle et textures

Deux possibilités :

- Choisir un OBJ déjà présent dans le pack, y compris un modèle importé mais pas encore exporté.
- Sélectionner un OBJ externe, puis le ou les MTL et les images utilisés par ce modèle. Les fichiers peuvent provenir de dossiers différents : les boutons Ajouter complètent la sélection, ils ne remplacent pas les fichiers déjà choisis.

Le modèle contient la forme. Le MTL décrit les matériaux et leurs images. Les chemins choisis restent affichés ; chaque MTL ou image peut être retiré de la sélection. Le bouton de textures accepte plusieurs images PNG, JPEG, TGA ou BMP. Les extensions `.mtl` et `.mlt` sont proposées pour les matériaux.

Les copies importées sont organisées dans un dossier propre à la configuration, par exemple `models/vehicle_transit/`. CPM réécrit `mtllib` et les chemins des images dans les MTL. Les noms de matériaux sont conservés ; leurs références sont reliées même si leur casse diffère. Les chemins d’origine ne sont jamais suivis pour importer silencieusement d’autres fichiers : seules les ressources sélectionnées sont lues.

Si un MTL ou une image manque, le contrôle indique ce qu’il faut sélectionner. Deux fichiers sélectionnés ayant le même nom sont refusés pour éviter une association ambiguë. Les options complexes de chemins `map_*` ne sont pas prises en charge ; utiliser un export MTL avec des chemins simples.

**Importer sans matériaux ni textures** est un choix explicite : les références aux matériaux sont retirées uniquement de la copie importée. L’OBJ original, les MTL et les images restent intacts.

Limites : OBJ de 32 Mio, MTL de 4 Mio, image compressée de 32 Mio, dimensions d’image de 4096 × 4096, au plus 1024 ressources et 256 Mio de ressources préparées dans le pack. Le chargeur 3D conserve ses limites supplémentaires sur les triangles et les pixels décodés.

### Modèles glTF (DynamX bêta)

Cocher **Oui, j’utilise DynamX bêta (autoriser le glTF)** avant de sélectionner un `.gltf` ou de réutiliser un tel modèle pour une nouvelle configuration. Sans confirmation, l’ajout est bloqué. Pour un glTF externe, sélectionner ses buffers `.bin` et ses images ; ses matériaux sont déjà intégrés au JSON. Le GLB n’est pas proposé. Voir le [guide glTF et ses limites](gltf.md).

## 3. Propriétés

L’assistant utilise le catalogue actif, y compris les paramètres d’addons. Il explique les paramètres au survol et utilise les champs adaptés à leurs types. Les erreurs sont signalées en rouge. Le champ Model est rempli à partir du modèle choisi, plutôt que demandé comme chemin à saisir.

La masse et plusieurs propriétés physiques disposent de valeurs de départ. Elles ne sont pas déduites de l’OBJ et ne garantissent pas un comportement réaliste. Les paramètres facultatifs avec une valeur par défaut peuvent être affichés.

Si un addon demande encore des paramètres à compléter, le choix **Continuer en brouillon** permet de préparer un objet incomplet. Les erreurs du catalogue resteront à corriger dans l’éditeur avant l’export. Les validations indispensables de fichiers, chemins, positions et assemblages restent appliquées.

## 4. Assemblage

### Roues des véhicules et remorques

Choisir l’une des options :

- **Plus tard** : ne pas ajouter de roues ; l’assistant rappelle que le véhicule reste à compléter.
- **Roue du pack** : réutiliser une configuration `wheel_*.dynx`, sans la copier ni la modifier.
- **Importer une configuration** : sélectionner un `.dynx` externe et son nom de destination. Si son modèle n’est pas déjà dans le pack, sélectionner aussi l’OBJ et ses dépendances ; le champ Model de la copie sera réécrit.
- **Créer une roue** : choisir son OBJ et ses dépendances, puis régler ses dimensions, son freinage et sa suspension. Les valeurs fournies sont des exemples à adapter.

L’assistant propose quatre emplacements de départ. Chacun possède son nom, sa position et ses cases motrice, côté droit, directrice et frein à main. Il est possible d’ajouter ou de retirer des emplacements et de choisir une configuration différente pour chaque roue, par exemple entre l’avant et l’arrière.

Les références sont écrites avec le nom interne du pack et le nom de la configuration. Les modifications du nom interne encore en mémoire sont prises en compte. Les fichiers réutilisés ne sont pas dupliqués. Une référence locale ambiguë ou absente est signalée lors de la vérification.

**X** correspond au déplacement gauche/droite, **Y** à avant/arrière, **Z** à la hauteur dans la configuration. Les positions proposées ne sont pas calculées depuis le modèle : ajuster ensuite leur placement dans l’aperçu 3D.

`Radius` et `Width` concernent la roue physique ; `ScaleModifier` ajuste la taille de son OBJ. Les dimensions physiques doivent rester cohérentes, notamment `RimRadius ≤ Radius`.

### Moteur et sons

Le véhicule peut réutiliser un moteur du pack ou recevoir un moteur de départ avec puissance, régime maximal, freinage, courbe et rapports de vitesses. Les rapports sont déclarés dans l’ordre attendu par DynamX : marche arrière, point mort, puis trois marches avant. Ils doivent être adaptés à votre véhicule et testés dans le jeu.

La création directe d’un fichier de type Moteur utilise le même modèle de départ. Les sons peuvent être reliés à une configuration existante du pack ; l’assistant ne crée pas les événements audio ni leurs fichiers OGG.

### Siège, volant et collision

Pour les véhicules, ajouter éventuellement un siège conducteur et une pièce de volant. Le nom du volant doit correspondre à celui de sa pièce dans l’OBJ. Laisser sa Position vide pour conserver le placement automatique depuis le modèle.

Pour les véhicules, blocs et props, une boîte de collision simple peut être ajoutée. Le centre et les demi-dimensions sont modifiables. Ce choix désactive les collisions complexes pour utiliser la boîte déclarée. Vérifier ensuite son volume dans l’aperçu et avec les poignées de taille.

## 5. Vérification et création

L’assistant prépare le lot en arrière-plan, puis affiche les configurations et les ressources qui seront ajoutées. Les ressources importées sont lues et vérifiées avant la confirmation ; changer les originaux ensuite ne remplace pas ces copies préparées. Les modèles déjà présents dans le pack restent des ressources réutilisées, pas des copies réimportées.

Les boutons **Précédent** et **Suivant** conservent les valeurs saisies. Revenir en arrière invalide le lot préparé et le prochain contrôle utilise les nouveaux choix. Une erreur permet de corriger les choix et de recommencer. Aucun fichier n’est ajouté au pack simplement en arrivant au récapitulatif.

Cliquer **Créer dans l’éditeur** pour ajouter le lot entier : objet principal, éventuelles nouvelles configurations de roue et de moteur, OBJ, MTL et images. Si un chemin entre en conflit ou qu’une configuration ne peut pas être ajoutée, le lot entier est refusé et les modifications existantes sont conservées. CPM ne remplace pas automatiquement une configuration ou une ressource de même nom.

Après création, ouvrir **Aperçu 3D**, vérifier les matériaux, les roues, le volant et les collisions, puis ajuster leur placement. Utiliser ensuite **File → Exporter** pour choisir la destination et le format dossier, ZIP ou DNXPACK. Cela fonctionne aussi avec un pack source ZIP, DNXPACK ou RAR : l’import reste préparé en mémoire jusqu’à l’export.

**File → Annuler les ajouts et suppressions** retire aussi les ressources importées non exportées. Les dossiers et archives source restent inchangés tant qu’un export n’a pas été demandé. Le comportement en jeu nécessite toujours un test avec la version de DynamX et les addons visés.
