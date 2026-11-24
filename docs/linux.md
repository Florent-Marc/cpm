# CPM sous Linux x86_64

Télécharger `cpm-linux-x86_64.tar.gz`, puis extraire l’archive dans un dossier personnel où vous avez le droit d’écrire et lancer :

```sh
tar -xzf cpm-linux-x86_64.tar.gz
cd CPM
./cpm
```

La distribution est construite sur Ubuntu 22.04 (glibc 2.35). Elle cible les bureaux Linux x86_64 compatibles, sous X11 ou Wayland. L’aperçu 3D nécessite OpenGL 3.1 ou OpenGL ES 3 avec des pilotes fonctionnels. L’ouverture de fenêtres et l’aperçu 3D ne fonctionnent pas dans un terminal sans session graphique.

Sur Ubuntu/Debian, si une bibliothèque graphique ou le sélecteur de fichiers manque :

```sh
sudo apt install libxkbcommon-x11-0 libgl1 libegl1 xdg-desktop-portal xdg-desktop-portal-gtk
```

L’archive contient les définitions DynamX et BasicsAddon dans `properties/`. Les modifications de ces fichiers sont conservées. Les catalogues sont aussi intégrés dans le programme et sont créés si les fichiers sont absents.

Le fichier brut `cpm-linux-x86_64` est également disponible ; le rendre exécutable avec `chmod +x cpm-linux-x86_64`, puis le lancer. Les préférences et catalogues sont stockés à côté de l’exécutable : éviter un dossier système en lecture seule.

Le coffre de jetons OpenMPS utilise Secret Service lorsqu’un coffre de session est disponible et déverrouillé. Sinon, conserver le jeton uniquement pour la session. La détection des mises à jour ouvre la page de téléchargement GitHub ; le remplacement automatique de l’exécutable est réservé à Windows x64.

Les fichiers livrés décrivent les paramètres éditables ; CPM ne distribue pas les mods Minecraft DynamX et BasicsAddon. Les installer séparément dans le jeu pour utiliser leurs fonctionnalités.
