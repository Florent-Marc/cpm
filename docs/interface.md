# Interface CPM

- **Fichier** : création, import et ouverture récente, exports et gestion des brouillons.
- **Édition** : annuler et rétablir les modifications.
- **Outils** : définitions des paramètres, recherche, modèles, diagnostic, édition groupée et OpenMPS.
- **Aide** : documentation DynamX, guide CPM, signalement de bugs et recherche de mises à jour.

Le bouton **Exporter** reste accessible dans la barre principale. Il ouvre le diagnostic et la comparaison avant export comme la commande du menu Fichier. Les exports locaux fonctionnent sans OpenMPS.

## Édition

La navigation par configurations et sections reste à gauche, dans un panneau redimensionnable. Le fond contrasté indique la sélection. Un point repère les configurations modifiées ; un `!` et un compteur accompagnent les erreurs. Les groupes de sections restent dépliables.

Les formulaires alignent les libellés et donnent davantage de place aux champs. Les descriptions restent disponibles au survol ; les champs vides proposent un exemple ou la valeur par défaut lorsqu’elle est définie. Les régimes moteur indiquent l’unité tr/min. Les paramètres d’addons ne reçoivent pas d’unité supposée. Les erreurs et avertissements sont expliqués sous le champ concerné et accompagnés d’un symbole : la couleur n’est pas le seul indicateur.

**Ajouter un paramètre**, **Retirer** et **Gérer…** rendent les actions explicites. La suppression d’un paramètre reste annulable dans l’historique du fichier.

## Accueil

Sans fichier ouvert, l’accueil propose d’ouvrir une archive ou un dossier de pack, de créer un pack et de retrouver les packs récents. Les chemins des packs récents aident à distinguer des noms identiques.

## OpenMPS

Le serveur actif reste visible au-dessus des onglets :

- **Packs distants** : recherche, téléchargement du ZIP joueurs et récupération d’une copie éditable selon les droits du jeton.
- **Publier** : préparation, récapitulatif et confirmation de publication.
- **Serveurs** : ajout, modification, test et suppression des connexions, avec mémorisation facultative des jetons dans le coffre système.

Les onglets ne publient rien à eux seuls. Le changement de serveur et les actions d’édition restent bloqués pendant un transfert. Les détails techniques du serveur et les instructions de première configuration sont dépliables.

## Présentation

L’interface utilise un thème sombre commun, des espacements et hauteurs de champs réguliers, des arrondis cohérents et un accent bleu pour les actions principales. Les erreurs utilisent un texte clair et un symbole ; les informations secondaires restent discrètes.
