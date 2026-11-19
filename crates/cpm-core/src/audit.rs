use crate::{Document, Pack, ParameterCatalog, Severity};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug)]
pub struct PackIssue {
    pub path: PathBuf,
    pub section: Option<usize>,
    pub parameter: String,
    pub severity: Severity,
    pub message: String,
}
impl Pack {
    /// Resolves a pack-local asset using the same resource roots as the OBJ preview.
    pub fn resolve_asset(&self, config: &Path, value: &str) -> Result<PathBuf, String> {
        let value = value.trim().trim_matches('"').replace('\\', "/");
        let mut candidates = Vec::new();
        if let Some((namespace, relative)) = value.split_once(':') {
            let namespace = crate::pack::safe_archive_path(namespace)?;
            if namespace.components().count() != 1 {
                return Err("Espace de noms invalide".into());
            }
            let relative = crate::pack::safe_archive_path(relative)?;
            candidates.push(PathBuf::from("assets").join(&namespace).join(&relative));
            for folder in ["models", "textures", "sounds"] {
                candidates.push(
                    PathBuf::from("assets")
                        .join(&namespace)
                        .join(folder)
                        .join(&relative),
                );
            }
        } else {
            let relative = crate::pack::safe_archive_path(&value)?;
            candidates.push(relative.clone());
            if let Some(parent) = config.parent() {
                candidates.push(parent.join(&relative));
            }
            for folder in ["models", "textures", "sounds"] {
                candidates.push(PathBuf::from(folder).join(&relative));
            }
            for namespace in ["dynamxmod", "minecraft"] {
                candidates.push(PathBuf::from("assets").join(namespace).join(&relative));
                for folder in ["models", "textures", "sounds"] {
                    candidates.push(
                        PathBuf::from("assets")
                            .join(namespace)
                            .join(folder)
                            .join(&relative),
                    );
                }
            }
        }
        candidates
            .into_iter()
            .find(|p| self.contains_resource(p))
            .ok_or_else(|| format!("Ressource absente du pack : {value}"))
    }
    /// Read-only report including in-memory documents. External pack references are warnings.
    pub fn audit(
        &self,
        documents: &BTreeMap<PathBuf, Document>,
        catalog: &ParameterCatalog,
    ) -> Vec<PackIssue> {
        let metadata = documents
            .get(Path::new("pack_info.dynx"))
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| self.read_config(0));
        let name = metadata
            .ok()
            .map(|d| crate::PackMetadata::from_document(&d).name)
            .unwrap_or_default();
        let mut issues = Vec::new();
        for (index, path) in self.configs().iter().enumerate() {
            let document = documents
                .get(path)
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| self.read_config(index));
            let document = match document {
                Ok(d) => d,
                Err(error) => {
                    issues.push(PackIssue {
                        path: path.clone(),
                        section: None,
                        parameter: String::new(),
                        severity: Severity::Error,
                        message: error,
                    });
                    continue;
                }
            };
            issues.extend(
                catalog
                    .diagnostics(path, &document)
                    .into_iter()
                    .map(|d| PackIssue {
                        path: path.clone(),
                        section: d.section,
                        parameter: d.parameter,
                        severity: d.severity,
                        message: d.message,
                    }),
            );
            for p in document.properties() {
                let mut error = None;
                let mut severity = Severity::Error;
                if ["DefaultEngine", "DefaultSounds", "AttachedWheel"].contains(&p.key.as_str())
                    && !p.value.trim().is_empty()
                {
                    let reference = p.value.trim();
                    let (namespace, stem) = reference
                        .rsplit_once('.')
                        .map(|(n, s)| (Some(n), s))
                        .unwrap_or((None, reference));
                    if namespace.is_some_and(|n| !n.eq_ignore_ascii_case(&name)) {
                        severity = Severity::Warning;
                        error = Some(format!(
                            "Référence vers un pack externe non chargé : {reference}"
                        ));
                    } else {
                        let matches = self
                            .configs()
                            .iter()
                            .filter(|p| {
                                p.file_stem()
                                    .is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case(stem))
                            })
                            .count();
                        if matches != 1 {
                            error = Some(format!("Référence absente ou ambiguë : {reference}"));
                        }
                    }
                }
                if p.key == "Model" && !p.value.trim().is_empty() {
                    error = self.resolve_asset(path, &p.value).err();
                }
                if let Some(message) = error {
                    issues.push(PackIssue {
                        path: path.clone(),
                        section: p.section,
                        parameter: p.key.clone(),
                        severity,
                        message,
                    });
                }
            }
        }
        issues
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audit_uses_unsaved_references_and_distinguishes_external_packs() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(
            t.path().join("pack_info.dynx"),
            "PackName: Demo\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        std::fs::write(t.path().join("vehicle_car.dynx"),"Model: missing.obj\nDefaultEngine: Demo.engine_missing\nDefaultSounds: Other.sounds_a\n").unwrap();
        std::fs::write(t.path().join("car.obj"), "data").unwrap();
        let pack = Pack::open(t.path()).unwrap();
        let mut docs = BTreeMap::new();
        let issues = pack.audit(&docs, &ParameterCatalog::default());
        assert!(issues
            .iter()
            .any(|i| i.parameter == "Model" && i.severity == Severity::Error));
        assert!(issues
            .iter()
            .any(|i| i.parameter == "DefaultSounds" && i.severity == Severity::Warning));
        docs.insert(
            PathBuf::from("vehicle_car.dynx"),
            Document::parse("Model: car.obj\n").unwrap(),
        );
        std::fs::write(t.path().join("car.obj"), "data").unwrap();
        assert!(!pack
            .audit(&docs, &ParameterCatalog::default())
            .iter()
            .any(|i| i.parameter == "Model"));
        assert!(pack
            .resolve_asset(Path::new("vehicle_car.dynx"), "../escape")
            .is_err());
    }
}
