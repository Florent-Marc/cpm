//! Cœur métier de Content's Pack Modifier.
//!
//! Lecture et édition conservatrice des configurations DynamX UTF-8.
//! Aucune dépendance à une interface graphique.

mod document;
mod pack;
mod pack_info;
pub use pack::{configuration_path, Pack};

pub use document::{Document, EditError, ParseError, Property, ReadError, Section};
pub use pack_info::{PackInfo, ValidationError};

/// Version du cœur CPM.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod export;
pub use export::{ExportFormat, ExportOptions, PackMetadata};

mod parameters;
pub use parameters::{configuration_issues, has_parameter, parameter_catalog, ParameterSpec};

mod catalog;
pub use catalog::{
    validate_catalog_file, CatalogFile, CatalogLoad, CatalogRule, CatalogStore, Diagnostic,
    NamedCatalogFile, ParameterCatalog, ParameterDefinition, Severity, VariableType,
};

mod model;
pub use model::{CollisionWire, ObjBatch, ObjMaterial, ObjMesh, ObjPart, ObjRequest, ObjTexture};

mod scene;
