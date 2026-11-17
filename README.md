# Content's Pack Modifier — CPM

Éditeur de packs [DynamX](https://wiki.dynamx.fr/fr/) en Rust, avec une interface
de bureau egui. Cette branche contient uniquement la version Rust.

## Fonctionnalités

- Import de dossiers de packs et d'archives ZIP, DNXPACK ou RAR non chiffrées.
- Navigation par catégories et sections, avec conservation des modifications.
- Ajout, modification et suppression de paramètres ; signalement des champs
  obligatoires connus et contrôle avant export.
- Création de packs et configurations ; bouton crayon pour créer, importer ou supprimer des fichiers `.dynx`.
- Export complet en dossier, ZIP ou DNXPACK, avec métadonnées et remplacement
  explicite d'une sortie existante.
- Liens vers le wiki et les rapports de bugs, version affichée et vérification
  des nouvelles releases Rust.

Le bouton **Aperçu 3D**, à côté du nom de la configuration, affiche les modèles OBJ présents dans le pack avec rotation, zoom et déplacement. Les matériaux MTL et leurs textures diffuses sont affichés ; la liste des pièces permet de les masquer ou de les isoler. Les animations et variantes DynamX restent à ajouter. Le catalogue des paramètres n'est pas un validateur exhaustif de tous les addons DynamX.

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
