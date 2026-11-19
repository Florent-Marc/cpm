use cpm_core::Document;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

pub struct Editor {
    pub document: Document,
    pub values: Vec<String>,
    pub is_pack_info: bool,
    pub path: std::path::PathBuf,
    pub catalog: std::sync::Arc<cpm_core::ParameterCatalog>,
    pub custom_key: String,
    pub custom_value: String,
    pub custom_parameter: Option<Option<usize>>,
    pub pending_delete: Option<usize>,
    pub pending_add: Option<(Option<usize>, String, String)>,
    saved_source: String,
    recorded: String,
    undo: Vec<String>,
    redo: Vec<String>,
}

impl Editor {
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|error| format!("Ouverture : {error}"))?;
        let document = Document::read_from(file).map_err(|error| error.to_string())?;
        Ok(Self::from_document(document, path))
    }

    pub fn from_document(document: Document, path: &Path) -> Self {
        let values = document
            .properties()
            .iter()
            .map(|p| p.value.clone())
            .collect();
        let saved_source = document.source().to_owned();
        Self {
            document,
            values,
            recorded: saved_source.clone(),
            undo: Vec::new(),
            redo: Vec::new(),
            saved_source,
            path: path.to_owned(),
            catalog: std::sync::Arc::default(),
            custom_key: String::new(),
            custom_value: String::new(),
            custom_parameter: None,
            pending_delete: None,
            pending_add: None,
            is_pack_info: path
                .file_name()
                .is_some_and(|name| name == "pack_info.dynx"),
        }
    }
    pub fn add_parameter(
        &mut self,
        section: Option<usize>,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        self.document
            .add_property(section, key, value)
            .map_err(|e| e.to_string())?;
        self.values = self
            .document
            .properties()
            .iter()
            .map(|p| p.value.clone())
            .collect();
        Ok(())
    }
    pub fn remove_parameter(&mut self, index: usize) -> Result<(), String> {
        self.document
            .remove_property(index)
            .map_err(|e| e.to_string())?;
        self.values = self
            .document
            .properties()
            .iter()
            .map(|p| p.value.clone())
            .collect();
        Ok(())
    }
    pub fn checkpoint(&mut self) {
        if self.recorded != self.document.source() {
            self.undo.push(std::mem::replace(
                &mut self.recorded,
                self.document.source().into(),
            ));
            self.redo.clear();
            while self.undo.len() > 100
                || self.undo.iter().map(String::len).sum::<usize>() > 16 * 1024 * 1024
            {
                self.undo.remove(0);
            }
        }
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.recorded != self.document.source()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn undo(&mut self) -> bool {
        self.checkpoint();
        let Some(source) = self.undo.pop() else {
            return false;
        };
        self.redo.push(self.recorded.clone());
        self.restore(source);
        true
    }
    pub fn redo(&mut self) -> bool {
        if self.recorded != self.document.source() {
            self.checkpoint();
            return false;
        }
        let Some(source) = self.redo.pop() else {
            return false;
        };
        self.undo.push(self.recorded.clone());
        self.restore(source);
        true
    }
    fn restore(&mut self, source: String) {
        self.document = Document::parse(&source).expect("historique valide");
        self.recorded = source;
        self.values = self
            .document
            .properties()
            .iter()
            .map(|p| p.value.clone())
            .collect();
        self.custom_parameter = None;
        self.pending_add = None;
        self.pending_delete = None;
    }
    pub fn apply_gizmo(&mut self, edit: &crate::preview::GizmoEdit) -> Result<(), String> {
        if self.document.source() != edit.expected_source {
            return Err("Configuration modifiée pendant le déplacement ; attendre l’actualisation puis réessayer".into());
        }
        if self.document.sections().get(edit.section).is_none() {
            return Err("Section supprimée".into());
        }
        self.checkpoint();
        if let Some(index) = self
            .document
            .properties()
            .iter()
            .position(|p| p.section == Some(edit.section) && p.key == edit.key)
        {
            self.document
                .set_value(index, &edit.value)
                .map_err(|e| e.to_string())?;
            self.values = self
                .document
                .properties()
                .iter()
                .map(|p| p.value.clone())
                .collect();
        } else {
            self.add_parameter(Some(edit.section), &edit.key, &edit.value)?;
        }
        self.checkpoint();
        Ok(())
    }
    pub fn issues(&self) -> Vec<String> {
        self.catalog.errors(&self.path, &self.document)
    }

    pub fn warnings(&self) -> Vec<String> {
        self.catalog
            .diagnostics(&self.path, &self.document)
            .into_iter()
            .filter(|d| d.severity == cpm_core::Severity::Warning)
            .map(|d| d.to_string())
            .collect()
    }

    pub fn is_dirty(&self) -> bool {
        self.document.source() != self.saved_source
    }
    pub fn revert(&mut self) {
        self.document = Document::parse(&self.saved_source).expect("source déjà analysée");
        self.values = self
            .document
            .properties()
            .iter()
            .map(|p| p.value.clone())
            .collect();
    }
    /// Refuse d'écraser un chemin existant, y compris un lien symbolique.
    pub fn save_copy(&mut self, path: &Path) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("Création de la copie : {error}"))?;
        self.document
            .write_to(&mut file)
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("Écriture incomplète de la copie : {error}"))?;
        self.saved_source = self.document.source().to_owned();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edit_save_reopen_preserves_original_and_refuses_overwrite() {
        let directory = std::env::temp_dir().join(format!("cpm-editor-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let original = directory.join("pack_info.dynx");
        let copy = directory.join("copy.dynx");
        let source = "// test\r\nPackName: Original\r\nDcFileVersion: 12.5.0\r\n";
        std::fs::write(&original, source).unwrap();
        let mut editor = Editor::open(&original).unwrap();
        assert!(editor.is_pack_info);
        editor.document.set_value(0, "Changed").unwrap();
        assert!(editor.is_dirty());
        assert!(editor.save_copy(&original).is_err());
        assert!(editor.is_dirty());
        editor.save_copy(&copy).unwrap();
        assert!(!editor.is_dirty());
        assert_eq!(std::fs::read_to_string(&original).unwrap(), source);
        assert_eq!(
            Editor::open(&copy).unwrap().document.properties()[0].value,
            "Changed"
        );
        editor.document.set_value(0, "Abandoned").unwrap();
        editor.revert();
        assert_eq!(editor.document.properties()[0].value, "Changed");
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;
    #[test]
    fn history_restores_structure_values_and_saved_state() {
        let mut e = Editor::from_document(
            Document::parse("// comment\r\nName: A\r\n").unwrap(),
            Path::new("vehicle_a.dynx"),
        );
        e.document.set_value(0, "B").unwrap();
        e.checkpoint();
        e.add_parameter(None, "Mass", "2").unwrap();
        e.checkpoint();
        assert!(e.undo());
        assert_eq!(e.values, vec!["B"]);
        assert!(e.undo());
        assert!(!e.is_dirty());
        assert!(e.redo());
        assert_eq!(e.values, vec!["B"]);
        e.document.set_value(0, "C").unwrap();
        e.checkpoint();
        assert!(!e.can_redo());
        assert!(e.document.source().starts_with("// comment\r\n"));
    }
}

#[cfg(test)]
mod gizmo_tests {
    use super::*;
    #[test]
    fn gizmo_applies_once_and_rejects_stale_document() {
        let mut e = Editor::from_document(
            Document::parse("Shape_body{\nPosition: 0 0 0\nScale: 1 2 3\n}\n").unwrap(),
            Path::new("vehicle_car.dynx"),
        );
        let edit = crate::preview::GizmoEdit {
            section: 0,
            key: "Position".into(),
            value: "2 0 0".into(),
            expected_source: e.document.source().into(),
        };
        e.apply_gizmo(&edit).unwrap();
        assert_eq!(e.values[0], "2 0 0");
        assert!(e.apply_gizmo(&edit).is_err());
        assert!(e.undo());
        assert_eq!(e.values[0], "0 0 0");
        assert!(e.redo());
        assert_eq!(e.values[0], "2 0 0");
    }
}
