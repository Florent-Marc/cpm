# Comparer avant export

Avec un pack ouvert, choisir **File → Exporter**, puis corriger les problèmes ou choisir explicitement **Forcer l’export** pour continuer malgré les erreurs de configuration.

Régler la destination, le format, le nom de sortie et les métadonnées. Cliquer sur **Comparer les modifications avant export** : CPM prépare une copie temporaire en arrière-plan, puis affiche les fichiers ajoutés, supprimés ou modifiés par rapport au pack source. Les modèles, textures et sons importés font partie de cette liste. Pour les configurations, déplier une ligne pour voir les paramètres avant/après, leur section et leur occurrence si des noms sont dupliqués. Un changement limité au texte, aux commentaires ou à une structure non analysable est indiqué séparément.

La comparaison inclut les métadonnées et les références internes connues adaptées par le changement de nom du pack. Elle concerne le pack source, pas un ancien fichier qui serait déjà présent à la destination. Elle ne présente pas une comparaison des pixels ou de la géométrie des ressources binaires.

**Exporter la copie comparée** devient disponible après préparation. Les fichiers sont figés : une modification externe du pack source après comparaison ne remplace pas cette copie. Changer une option d’export impose une nouvelle comparaison. Fermer la fenêtre abandonne la copie préparée ; pour inclure des changements externes, recommencer la comparaison.

Les contrôles de destination, chemins et remplacement restent obligatoires, même pour un export forcé. Une sortie apparue entre la préparation et l’export n’est pas remplacée sans l’option prévue. L’export classique reste indépendant d’OpenMPS.
