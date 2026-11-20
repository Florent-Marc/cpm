# API compagnon pour OpenMPS

Cette intégration contient trois commits serveur dans [`openmps-api-v1.patch`](openmps-api-v1.patch), préparés pour [gabidut/OpenMPS](https://github.com/gabidut/OpenMPS) à partir du commit `d475044df956cc0d3a818d92f993dcef39fdb192` :

1. API d’administration avec jetons révocables, publication de versions immuables, génération des archives et routes de ressources.
2. Documentation `API.md` et tests HTTP avec un serveur PHP réel.
3. Récupération des packs éditables et de leurs ressources originales, réservée aux jetons disposant du droit `recover`. La suite comprend sept tests HTTP.

Les changements sont proposés dans la [PR OpenMPS nº 1](https://github.com/gabidut/OpenMPS/pull/1), depuis la branche `Florent-Marc/OpenMPS:cpm-api-v1`. Aucun serveur n’a été déployé. Les exports locaux de CPM restent indépendants de cette API.

Dans une copie propre du dépôt OpenMPS, créer une branche et appliquer :

```sh
git switch -c cpm-api-v1 d475044df956cc0d3a818d92f993dcef39fdb192
git am /chemin/vers/cpm/integrations/openmps/openmps-api-v1.patch
python3 tests/test_api.py
```

Si le dépôt a évolué, appliquer sur une branche dédiée et résoudre les éventuels conflits avant de fusionner. Le patch ne contient ni jeton, ni packs utilisateurs, ni binaire PHP. Il conserve les fichiers du dépôt non concernés, notamment le chargeur Java fourni par OpenMPS.

Lire ensuite `API.md` dans OpenMPS pour configurer l’URL publique, le serveur HTTP et créer le jeton. Le droit `recover` doit être accordé explicitement ; les jetons existants ne le reçoivent pas automatiquement. La récupération restitue les originaux conservés sur le serveur, sans reconstruire les fichiers manquants ou chiffrés.

Le [guide CPM](../../docs/openmps.md) décrit l’utilisation et les limites.

Pour tester les deux projets ensemble avec Rust/Cargo et PHP OpenSSL/ZipArchive disponibles :

```sh
OPENMPS_SOURCE=/chemin/vers/OpenMPS python3 integrations/openmps/test_roundtrip.py
```

Ce test utilise uniquement une copie temporaire du serveur, des identifiants temporaires et une adresse loopback. Aucun compte réel n’est utilisé. Les tests standards `cargo test --workspace` n’effectuent pas cette connexion ; le test HTTP ignoré par défaut est lancé explicitement par ce script.
