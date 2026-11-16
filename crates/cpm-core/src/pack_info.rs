use crate::Document;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub property: &'static str,
    pub message: &'static str,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} : {}", self.property, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Vue validée des deux champs obligatoires documentés de pack_info.dynx.
/// Le Document d'origine reste la source à sauvegarder, pour garder les addons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackInfo<'a> {
    pub pack_name: &'a str,
    pub dc_file_version: &'a str,
}

impl<'a> PackInfo<'a> {
    pub fn from_document(document: &'a Document) -> Result<Self, Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut required = |key| {
            let mut matches = document
                .properties()
                .iter()
                .filter(|p| p.section.is_none() && p.key == key);
            let first = matches.next();
            let message = if matches.next().is_some() {
                Some("propriété dupliquée à la racine")
            } else if first.is_none_or(|p| p.value.is_empty()) {
                Some("propriété obligatoire absente ou vide")
            } else {
                None
            };
            if let Some(message) = message {
                errors.push(ValidationError {
                    property: key,
                    message,
                });
            }
            first.map(|p| p.value.as_str()).unwrap_or("")
        };
        let pack_name = required("PackName");
        let dc_file_version = required("DcFileVersion");
        if errors.is_empty() {
            Ok(Self {
                pack_name,
                dc_file_version,
            })
        } else {
            Err(errors)
        }
    }
}
