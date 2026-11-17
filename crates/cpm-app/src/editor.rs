use cpm_core::Document;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

pub struct Editor {
    pub document: Document,
    pub values: Vec<String>,
    pub is_pack_info: bool,
    pub path: std::path::PathBuf,
    pub custom_key: String,
    pub custom_value: String,
    pub custom_parameter: Option<Option<usize>>,
    pub pending_delete: Option<usize>,
    pub pending_add: Option<(Option<usize>, String, String)>,
    saved_source: String,
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
            saved_source,
            path: path.to_owned(),
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
    pub fn issues(&self) -> Vec<String> {
        cpm_core::configuration_issues(&self.path, &self.document)
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
