//! Cœur métier de Content's Pack Modifier.
//!
//! Cette crate accueillera les modèles de packs DynamX, leur lecture,
//! leur validation et leur sauvegarde.
//! Elle ne doit pas dépendre de l'interface graphique.

/// Version du cœur CPM.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
