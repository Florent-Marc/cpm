# Modèles glTF et DynamX bêta

L’assistant de création accepte maintenant les modèles **glTF 2.0 JSON (`.gltf`)**, en plus des OBJ. Dans le sélecteur de modèle (objet principal ou nouvelle roue), lire l’avertissement et cocher **Oui, j’utilise DynamX bêta (autoriser le glTF)**. Sans cette confirmation, le sélecteur reste limité aux OBJ, les glTF déjà présents ne peuvent pas être choisis pour une nouvelle configuration et la préparation refuse un glTF. Décocher la case bloque à nouveau sa création. Cette confirmation est déclarative : CPM ne détecte pas votre installation de Minecraft.

La branche `beta` de DynamX reconnaît `.gltf` dans `DxModelPath` ; son fonctionnement peut différer d’une release stable. L’import ne propose pas GLB (`.glb`), que ce sélecteur de formats de DynamX ne reconnaît pas. Ne pas simplement renommer un GLB en glTF.

## Importer un modèle

1. Dans **File → Créer**, choisir un objet avec modèle.
2. À l’étape **Modèle et textures**, confirmer DynamX bêta, puis **Choisir un fichier OBJ ou glTF…**.
3. Sélectionner le `.gltf`. Ajouter les fichiers `.bin` référencés avec **Ajouter les buffers BIN…**, puis les images PNG/JPEG avec **Ajouter des textures…**. Les matériaux sont déjà dans le glTF : pas de MTL à ajouter.
4. Les buffers et images intégrés en base64 ne nécessitent pas de sélection séparée. Une dépendance externe absente bloque la préparation et indique le fichier à sélectionner.
5. Continuer l’assistant et vérifier le récapitulatif, puis **Créer dans l’éditeur** et **File → Exporter**.

Seuls les fichiers explicitement sélectionnés sont lus lors de l’import. CPM prépare `model.gltf`, les buffers et les images dans un dossier propre à la configuration, réécrit leurs URI dans la copie et conserve les originaux. Des fichiers sélectionnés de même nom sont refusés plutôt que reliés de manière ambiguë. Les URI réseau et chemins absolus sont refusés ; aucun téléchargement de dépendance n’est effectué.

## Aperçu et export

**Aperçu 3D** et **Outils → Modèles** reconnaissent les `.gltf` présents dans un dossier ou une archive. L’affichage statique tient compte des pièces/nœuds nommés, de leur hiérarchie et de leurs transformations, des triangles, UV et textures/couleurs de base. Les roues, le volant et les boîtes de collision utilisent le même assemblage que les OBJ. Le déplacement des collisions reste disponible.

Les animations, le skinning, les morphs, le rendu PBR complet, les matériaux avancés et les variantes glTF ne sont pas simulés. Les normales de face sont calculées pour l’aperçu. Les primitives autres que triangles sont ignorées avec un avertissement. Les accessors sparse et extensions obligatoires (notamment Draco/Meshopt) sont refusés : exporter une géométrie triangulée sans compression ni extension obligatoire. Ces limites concernent la preview ; le fichier glTF importé conserve ses données, notamment les animations, pour DynamX.

Les sorties **Dossier**, **ZIP** et **DNXPACK** conservent le glTF et ses dépendances. L’export OpenMPS continue à préparer une copie distincte et à signaler les références glTF à vérifier après déplacement. Toujours tester le pack avec la bêta de DynamX visée.

Limites : glTF de 32 Mio, buffer de 128 Mio, image compressée de 32 Mio et 4096 × 4096 maximum, ressources lues de 256 Mio, textures décodées de 256 Mio, un million de triangles. Les URI et les plages d’accès aux buffers sont vérifiées avant le chargement.
