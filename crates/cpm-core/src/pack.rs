use crate::Document;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_CONFIG_BYTES: u64 = 8 * 1024 * 1024;

/// Index des configurations d'un dossier ou d'une archive, sans extraction.
/// Les ressources (modèles, textures…) ne sont pas chargées en mémoire.
#[derive(Debug)]
pub struct Pack {
    source: PathBuf,
    configs: Vec<PathBuf>,
    archive_indices: Option<BTreeMap<PathBuf, usize>>,
}

impl Pack {
    pub fn open(path: &Path) -> Result<Self, String> {
        let source = fs::canonicalize(path).map_err(|e| e.to_string())?;
        let mut configs = Vec::new();
        let archive_indices = if source.is_dir() {
            let mut pending = vec![source.clone()];
            while let Some(directory) = pending.pop() {
                for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let kind = entry.file_type().map_err(|e| e.to_string())?;
                    // Ne pas suivre de liens vers d'autres packs ou des cycles.
                    if kind.is_dir() {
                        pending.push(entry.path());
                    } else if kind.is_file() && is_config(&entry.path()) {
                        configs.push(
                            entry
                                .path()
                                .strip_prefix(&source)
                                .map_err(|e| e.to_string())?
                                .to_owned(),
                        );
                    }
                }
            }
            None
        } else {
            let mut archive = zip::ZipArchive::new(File::open(&source).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let mut indices = BTreeMap::new();
            for index in 0..archive.len() {
                let entry = archive.by_index_raw(index).map_err(|e| e.to_string())?;
                let name = entry
                    .enclosed_name()
                    .ok_or("Chemin non sûr dans l'archive")?;
                if !entry.is_dir() && is_config(&name) {
                    if entry
                        .unix_mode()
                        .is_some_and(|mode| mode & 0o170000 == 0o120000)
                    {
                        return Err("Une configuration de l'archive est un lien symbolique".into());
                    }
                    if indices.insert(name.clone(), index).is_some() {
                        return Err(format!("Configuration dupliquée : {}", name.display()));
                    }
                    configs.push(name);
                }
            }
            Some(indices)
        };
        if !configs.iter().any(|p| p == Path::new("pack_info.dynx")) {
            return Err("Le pack doit contenir pack_info.dynx à sa racine".into());
        }
        configs.sort_by(|a, b| {
            let rank = |p: &PathBuf| {
                if p == Path::new("pack_info.dynx") {
                    0
                } else {
                    1
                }
            };
            rank(a).cmp(&rank(b)).then_with(|| a.cmp(b))
        });
        Ok(Self {
            source,
            configs,
            archive_indices,
        })
    }

    /// Chemins relatifs, ordonnés avec pack_info.dynx en premier.
    pub fn configs(&self) -> &[PathBuf] {
        &self.configs
    }
    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn read_config(&self, index: usize) -> Result<Document, String> {
        let path = self.configs.get(index).ok_or("Configuration inconnue")?;
        let source = if let Some(indices) = &self.archive_indices {
            let mut archive =
                zip::ZipArchive::new(File::open(&self.source).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let entry = archive.by_index(indices[path]).map_err(|e| e.to_string())?;
            if entry.enclosed_name().as_ref() != Some(path) {
                return Err("L'archive a changé : rouvrir le pack".into());
            }
            read_limited(entry)?
        } else {
            let full = fs::canonicalize(self.source.join(path)).map_err(|e| e.to_string())?;
            if !full.starts_with(&self.source) {
                return Err("La configuration sort du dossier du pack".into());
            }
            read_limited(File::open(full).map_err(|e| e.to_string())?)?
        };
        Document::parse(&source).map_err(|e| format!("{} : {e}", path.display()))
    }
}

fn is_config(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("dynx"))
}

fn read_limited(reader: impl Read) -> Result<String, String> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err("Configuration trop volumineuse (maximum 8 Mio)".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
