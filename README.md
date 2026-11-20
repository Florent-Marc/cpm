# Content's Pack Modifier — CPM

Éditeur de packs [DynamX](https://wiki.dynamx.fr/fr/) en Rust, avec une interface
de bureau egui. Cette branche contient uniquement la version Rust.

## Aide et documentation

Le [wiki CPM](https://github.com/Florent-Marc/cpm/wiki) accompagne les utilisateurs : premiers pas, création de packs, aperçu 3D et collisions, propriétés et addons, exports et OpenMPS. Dans l’application, **Aide CPM** ouvre ces guides et **Wiki** ouvre la documentation DynamX.

Le [guide de l’interface](docs/interface.md) présente les menus, l’accueil et les onglets OpenMPS.

## Fonctionnalités

- Import de dossiers de packs et d'archives ZIP, DNXPACK ou RAR non chiffrées.
- Navigation par catégories et sections, recherche dans le pack et historique annuler/rétablir par fichier.
- Champs adaptés aux types, descriptions au survol, diagnostic cliquable des paramètres et références.
- Ajout, modification et suppression de paramètres ; signalement des champs
  obligatoires connus et contrôle avant export.
- Assistant de création par étapes : import OBJ/MTL/textures, propriétés expliquées, roues existantes ou nouvelles, moteur de départ, siège et collision ; bouton crayon pour gérer les fichiers `.dynx`.
- Import et aperçu statique glTF 2.0, avec confirmation DynamX bêta obligatoire dans l’assistant ([guide](docs/gltf.md)).
- Diagnostic avant export : corriger les configurations ou forcer explicitement l’export malgré leurs erreurs.
- [Comparaison avant export](docs/export-review.md) et [édition groupée](docs/bulk-edit.md) avec récapitulatif avant application.
- Récupération d’une copie éditable depuis OpenMPS avec le droit d’administration `recover`.
- Export complet en dossier, ZIP ou DNXPACK, avec métadonnées et remplacement
  explicite d'une sortie existante.
- Publication facultative sur OpenMPS : gestion de plusieurs serveurs, jetons séparés avec mémorisation dans le coffre système, diagnostic sur copie, versions distantes et ZIP joueurs vérifié. Voir le [guide OpenMPS](docs/openmps.md).
- Liens vers le wiki et les rapports de bugs, version affichée et vérification
  des nouvelles releases Rust.

Le bouton **Aperçu 3D**, à côté du nom de la configuration, affiche les modèles OBJ présents dans le pack avec rotation, zoom et déplacement. Les matériaux MTL et leurs textures diffuses sont affichés ; la liste des pièces permet de les masquer ou de les isoler. L’aperçu se met à jour après les modifications, assemble les roues et le volant, affiche les boîtes de collision en fil de fer et propose les variantes diffuses DynamX. La sélection 3D rejoint la configuration ; les flèches permettent de déplacer les pièces et redimensionner les boîtes. Les animations restent à ajouter. Le catalogue des paramètres n'est pas un validateur exhaustif de tous les addons DynamX.

Les nouvelles commandes sont regroupées dans **Outils** : **Recherche**, **Modèles**, **Diagnostic**. Voir le [guide des outils d’édition](docs/editor-tools.md).

Voir aussi le [guide de création pas à pas](docs/creation-wizard.md).

## Démarrer

Installer Rust stable (1.95 ou supérieur). Un compilateur C++ est également
nécessaire pour la bibliothèque de lecture RAR ; sous Windows, utiliser les
outils Visual Studio C++ et le Windows SDK. Java et Maven ne sont pas nécessaires.

```sh
cargo run -p cpm-app
```

Pour compiler une version optimisée :

```sh
cargo build -p cpm-app --release
```

L'exécutable se trouve dans `target/release/cpm.exe` sur Windows, et
`target/release/cpm` sur Linux. Les sélecteurs Linux nécessitent un portail XDG.

## Structure

| Dossier | Rôle |
| --- | --- |
| `crates/cpm-core` | Lecture, édition, validation et export des packs |
| `crates/cpm-app` | Interface egui, sélecteurs et mises à jour |
| `docs/rust.md` | Guide détaillé, limites et fonctionnement des exports |
| `.github/workflows` | Vérification, compilation et publication Windows |

Le core ne dépend pas de l'interface. Les icônes, ressources Windows et le
script de mise à jour sont inclus dans `crates/cpm-app/assets`.

## Vérifier

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Voir le [guide détaillé](docs/rust.md) pour les formats pris en charge, les
règles de validation et la publication des releases utilisées par l'updater.

## Auteur

[@MK](https://github.com/Florent-Marc)

Le bouton **Propriétés** permet de personnaliser les types, obligations, sévérités et valeurs par défaut. Les définitions sont enregistrées dans `properties/` à côté de l’exécutable ; des fichiers JSON supplémentaires permettent de prendre en charge les addons. Voir [le guide](docs/rust.md#catalogue-de-propriétés-et-addons).

## Version Windows à télécharger

Les [releases GitHub](https://github.com/Florent-Marc/cpm/releases) proposent un installateur Windows 10/11 x64 (`cpm-setup-windows-x86_64.exe`) et une version portable (`cpm-windows-x86_64.exe`). Java et Rust ne sont pas nécessaires pour utiliser ces exécutables. Le dossier `properties/` et l’historique sont enregistrés à côté du programme.
