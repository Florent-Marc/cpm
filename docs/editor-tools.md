# Outils d’édition du pack

Cette branche contient dix améliorations, chacune dans son propre commit. Elles ne modifient les fichiers source qu’à travers un export explicite. Les packs RAR restent exportables en dossier, ZIP ou DNXPACK.

| Commande | Utilisation |
| --- | --- |
| Édition → Annuler / Rétablir | Historique des valeurs, ajouts et suppressions de paramètres du fichier courant |
| Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z | Historique du document, hors saisie des autres fenêtres d’outils |
| Champs typés | Booléens en cases à cocher, énumérations en listes, vecteurs en composantes X/Y/Z, quaternions W/X/Y/Z |
| Outils → Recherche | Recherche sans distinction de casse dans les noms de configurations, sections, paramètres et valeurs |
| Outils → Diagnostic | Contrôle du catalogue et des références locales ; une ligne ouvre le fichier, la section et rejoint le paramètre |
| File → Exporter | Diagnostic avant les options d’export ; corriger les erreurs, puis relancer le contrôle |
| Outils → Modèles | Arbre des OBJ du pack, y compris ceux sans configuration ; aperçu brut ou assemblé depuis une configuration |
| Aperçu → Actualisation automatique | Rechargement après environ 350 ms de pause de saisie, avec conservation de la caméra et des visibilités |
| Aperçu → Pièces | Arbre des pièces du modèle principal et des roues ; afficher, masquer, isoler ou sélectionner |
| Clic sur le modèle ou une arête de collision | Sélection de la pièce visible au premier plan, puis ouverture de sa section de configuration |
| Sélection d’une section dans l’éditeur | Repérage de la roue, du volant ou de la boîte correspondante en orange dans l’aperçu |
| Sélecteur de variante | Variantes diffuses de `MaterialVariants` ou de l’ancien champ `Textures` |
| Déplacer / Taille (boîte) | Poignées X/Y/Z pour modifier une Position ou les demi-dimensions d’une boîte Shape… |

## Historique et champs

L’historique est indépendant pour chaque fichier et reste disponible en changeant de configuration. Il conserve jusqu’à 100 états précédents, avec un budget de 16 Mio. Une nouvelle modification efface les états « rétablir ». Les créations et suppressions de fichiers utilisent toujours les commandes de gestion du pack, pas cet historique.

Une valeur numérique incomplète peut être conservée pendant la frappe, mais reste contrôlée avant export. Les valeurs importées invalides ou avec un format inhabituel restent éditables en texte afin de pouvoir les corriger. Aucun champ n’est normalisé simplement en l’affichant.

## Recherche et diagnostic

La recherche utilise les documents en mémoire et lit les autres configurations en arrière-plan, après une pause de saisie de 300 ms. Elle affiche au maximum 500 résultats. Relancer la requête pour actualiser les résultats après d’autres modifications.

Le diagnostic est un instantané en lecture seule du pack et des documents en mémoire. Il vérifie les erreurs et avertissements du catalogue, les ressources Model et les références DefaultEngine, DefaultSounds et AttachedWheel. Une référence dans le pack doit désigner une configuration unique ; un autre pack non chargé est signalé en avertissement, car sa présence dans le jeu ne peut pas être vérifiée. Ce contrôle n’est pas un validateur exhaustif des ressources audio, des addons, des variantes ou de Minecraft.

Les erreurs doivent être corrigées avant de continuer vers l’export. Les avertissements permettent de continuer. La vérification déjà présente dans le moteur d’export reste active. Le diagnostic ne charge pas la géométrie ni toutes les images d’un pack.

## Arbre et aperçu

L’arbre indexe les chemins OBJ et les configurations qui les utilisent. Il n’ouvre leurs textures et géométries qu’à la sélection. Les variantes et les réglages de pièce nécessitent l’ouverture depuis une configuration ; l’aperçu OBJ seul est en lecture seule. Le navigateur doit être rouvert pour actualiser son index après des changements de Model. Les recherches et sélections ne naviguent pas pendant un export ou un diagnostic modal.

Un seul modèle est analysé à la fois ; une demande en attente plus récente remplace la précédente. Les chargements dépassés ne remplacent pas le modèle courant. Les limites de triangles, textures et tailles de fichiers du chargeur restent appliquées. Les changements de fichiers sur disque hors CPM ne sont pas surveillés.

Le clic 3D utilise les triangles visibles ; les arêtes de collision affichées sont prioritaires à proximité du pointeur. Les parties sans section associée renvoient à la racine. Le rendu reste statique : pas de simulation physique ni de braquage animé.

## Variantes diffuses

Exemple de configuration moderne :

```text
MaterialVariants{
    BaseMaterial: paint
    Variants: blue green
}
```

Le sélecteur propose `paint`, `blue`, `green`. Le matériau de base peut être remplacé par un matériau du MTL du même nom que la variante. Le MTL peut aussi déclarer des textures diffuses nommées :

```text
newmtl paint
Kd 1 1 1
map_Kd red.png default
map_Kd blue.png blue
```

Sans texture correspondante, le matériau conserve sa texture diffuse par défaut. Les noms sont comparés sans distinction de casse. Le choix de variante est un réglage d’aperçu et ne réécrit pas la configuration. Les roues utilisent leur propre catalogue de variantes et le même indice, avec retour à leur variante par défaut si l’indice n’existe pas. Les normal maps, shaders Minecraft et variantes indépendantes des feux ne sont pas simulés.

## Déplacer et redimensionner

1. Sélectionner une roue, un volant ou une boîte dans la vue 3D, son arbre de pièces ou la liste des sections.
2. Choisir **Déplacer**, ou **Taille (boîte)** pour une Shape de type BOX.
3. Faire glisser l’extrémité d’une flèche. La valeur apparaît pendant le geste ; le relâchement l’applique à la configuration et actualise le modèle.
4. **Échap** annule le geste. **Ctrl+Z** annule une manipulation appliquée. La grille facultative arrondit la valeur au pas choisi.

Les flèches suivent les axes du fichier DynamX et tiennent compte de ScaleModifier. Un axe dirigé vers la caméra peut ne pas être manipulable : changer de vue. Les demi-dimensions restent positives (minimum 0,001). Les alias anciens Position/Scale sont conservés et les composantes non modifiées gardent leur texte d’origine.

Le déplacement nécessite une Position explicite valide. Un volant placé automatiquement depuis l’OBJ doit d’abord recevoir une Position adaptée dans sa configuration. L’outil ne transforme pas automatiquement un pivot implicite en pivot manuel. Si la configuration a changé pendant un geste, l’application refuse l’écriture basée sur l’ancien état : attendre l’actualisation puis recommencer.
