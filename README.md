
# Content's Pack Modifier

The CPM program was created to simplify the creation and modification of [Dynamx](https://dynamx.fr/) packs by consolidating everything into a Java application, which is still in development. This tool aims to streamline processes and enhance efficiency for users working with packs.


## In progress

Being still in development, the CPM program has some errors and unfinished features. However, I am open to discussions and pull requests so that, together, we can create a better tool to save us time.



## Lib

- json.simple for save data
- jimObjModelImporterJFX for render 3D
- javassist for addon import

## Authors

[@MK](https://github.com/Florent-Marc) 

## Rust workspace

A Rust workspace is available in this branch: see [the Rust guide](docs/rust.md).
`cpm-core` reads and edits UTF-8 DynamX configuration documents while preserving their text.
`cpm-app` opens individual .dynx configurations, pack folders, and .dnxpack/ZIP/RAR archives through File → Importer. It groups configurations and sections, retains edits while navigating, and exports full packs as folders, ZIP, or DNXPACK via File → Exporter, with metadata editing and explicit replacement. The 3D preview is not implemented yet.
