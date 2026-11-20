# Éditer plusieurs configurations

Ouvrir un pack puis **Outils → Édition groupée…**. Les modifications sont préparées en mémoire et n’écrivent dans le pack source qu’à travers un export explicite.

1. Rechercher un nom de fichier ou filtrer par préfixe de catégorie (`vehicle`, `engine`, `wheel`, `armor`…). Cocher les fichiers voulus, ou sélectionner tous les fichiers affichés. La sélection reste conservée lorsqu’on change de filtre ; **Tout désélectionner** la vide.
2. Laisser la section vide pour viser la racine, ou saisir un nom exact pour modifier toutes les sections de ce nom dans chaque fichier. Les fichiers où cette section manque sont signalés et ignorés.
3. Choisir **Modifier les présents**, **Ajouter les absents** ou **Supprimer**. L’ajout ne remplace pas une valeur déjà présente. Les alias connus du catalogue sont reconnus.
4. Saisir le paramètre et sa valeur. La liste propose les définitions du premier fichier sélectionné ; le contrôle est refait pour chacun des fichiers. Une valeur incompatible avec un type connu est refusée. Un paramètre personnalisé reste possible.
5. **Préparer les modifications** affiche, fichier par fichier, les valeurs avant/après et les erreurs de configuration restantes. Rien n’est encore appliqué. Modifier la sélection ou une option invalide ce récapitulatif.
6. Vérifier puis **Appliquer ces modifications aux fichiers sélectionnés**. Si une configuration a changé depuis la préparation, aucun fichier du lot n’est modifié et il faut préparer de nouveau.

Les commentaires et les paramètres non visés sont conservés. **Édition → Annuler / Rétablir** fonctionne dans chaque fichier modifié ; il ne s’agit pas d’un annuler global du lot. Utiliser ensuite **File → Exporter**, consulter la comparaison puis exporter la copie préparée. La suppression d’un obligatoire peut créer une erreur : elle est affichée dans le récapitulatif et reste à corriger ou à accepter explicitement lors d’un export forcé.
