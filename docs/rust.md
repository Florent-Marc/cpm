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

Le core n'a aucune dépendance externe. L'application utilise egui via eframe
avec le moteur OpenGL (`glow`). `Cargo.lock` est conservé dans Git.
Un environnement graphique de bureau et un pilote OpenGL sont nécessaires.

## Interface egui

`cargo run -p cpm-app` lance l'éditeur d'un fichier `.dynx` à la fois :

1. Saisir le chemin du fichier, puis cliquer sur **Ouvrir**.
2. Choisir la racine ou une section à gauche, puis modifier ses valeurs.
3. Consulter le texte résultant et les erreurs de métadonnées de pack.
4. Indiquer un chemin libre et cliquer sur **Enregistrer une copie**.

L'enregistrement refuse tout chemin existant. Pour une sauvegarde suivante,
choisir un autre nom. Une erreur d'écriture peut laisser une copie incomplète,
mais le fichier d'origine n'est pas modifié. **Annuler les modifications**
restaure le dernier état enregistré. L'ouverture d'un autre fichier est bloquée
si des modifications restent à enregistrer ; la fermeture demande de confirmer
leur abandon. Les chemins sont saisis manuellement dans cette première version.

Les dossiers de packs, les archives, le sélecteur de fichiers natif et la
prévisualisation 3D ne sont pas encore intégrés.

## Première API du core

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

Cette première version ne charge pas un dossier ou une archive `.dnxpack`,
n'interprète pas encore les champs des véhicules et ne gère pas l'ajout ou la
suppression de propriétés. La validation des versions, des références et des
types de valeurs reste à implémenter. `write_to` écrit dans un flux : l'appelant
doit gérer les sauvegardes atomiques et les copies de secours sur disque.

## Prochaines étapes

1. Ajouter des packs réels de référence et vérifier leurs cycles de sauvegarde.
2. Charger les dossiers de packs et détecter leurs fichiers de configuration.
3. Ajouter les vues métier (véhicules, roues, sièges) et leurs validations.
4. Ajouter les archives avec contrôle des chemins, puis l'interface et la 3D.
