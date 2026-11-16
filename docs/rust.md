# CPM en Rust

Cette branche prépare une réécriture progressive de CPM. Le workspace est
un squelette : il ne lit ni ne modifie encore de packs. Le code Java reste
disponible dans `src/` comme référence.

## Organisation

- `crates/cpm-core` : bibliothèque pour les modèles, la lecture, la validation,
  la sauvegarde et les archives DynamX.
- `crates/cpm-app` : point d'entrée de l'application, qui dépend de `cpm-core`.
  Il affiche pour l'instant un message dans le terminal. Aucune bibliothèque
  graphique n'est encore choisie.

Le core ne dépend pas de l'application et ne doit contenir aucun composant
d'interface. Il renverra des données et des erreurs que l'application affichera.

## Démarrer

Installer Rust stable avec Cargo, puis lancer depuis la racine du dépôt :

```sh
cargo check --workspace
cargo run -p cpm-app
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Le workspace n'a aucune dépendance externe. Cargo générera un `Cargo.lock`
à la première commande ; le conserver dans Git puisque le workspace contient
une application. Aucun test métier n'est encore implémenté.

## Prochaines étapes

1. Choisir des packs de référence et les comportements à préserver.
2. Implémenter leur lecture dans le core, avec des erreurs explicites.
3. Tester la conservation des données lors d'un cycle lecture/sauvegarde.
4. Ajouter l'interface et la prévisualisation 3D dans l'application.
