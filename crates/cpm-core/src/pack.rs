use crate::Document;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_CONFIG_BYTES: u64 = 8 * 1024 * 1024;

/// Index des configurations d'un dossier ou d'une archive, sans extraction.
/// Les ressources (modèles, textures…) ne sont pas chargées en mémoire.
#[derive(Debug, Clone)]
pub struct Pack {
    source: PathBuf,
    configs: Vec<PathBuf>,
    archive_indices: Option<BTreeMap<PathBuf, usize>>,
    rar: bool,
    resources: std::sync::Arc<BTreeMap<PathBuf, usize>>,
    added: BTreeMap<PathBuf, Document>,
    removed: std::collections::BTreeSet<PathBuf>,
}

impl Pack {
    pub fn open(path: &Path) -> Result<Self, String> {
        let source = fs::canonicalize(path).map_err(|e| e.to_string())?;
        let mut configs = Vec::new();
        let mut resources = BTreeMap::new();
        let rar = source
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rar"));
        let archive_indices = if source.is_dir() {
            let mut pending = vec![source.clone()];
            while let Some(directory) = pending.pop() {
                for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let kind = entry.file_type().map_err(|e| e.to_string())?;
                    // Ne pas suivre de liens vers d'autres packs ou des cycles.
                    if kind.is_dir() {
                        pending.push(entry.path());
                    } else if kind.is_file() {
                        let relative = entry
                            .path()
                            .strip_prefix(&source)
                            .map_err(|e| e.to_string())?
                            .to_owned();
                        resources.insert(relative.clone(), 0);
                        if is_config(&relative) {
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
            }
            None
        } else if rar {
            for entry in unrar::Archive::new(&source)
                .open_for_listing()
                .map_err(|e| e.to_string())?
            {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = safe_archive_path(&entry.filename.to_string_lossy())?;
                if entry.is_encrypted() || entry.is_split() {
                    return Err(
                        "Les RAR chiffrés ou multivolumes ne sont pas pris en charge".into(),
                    );
                }
                if entry.is_file() && resources.insert(name.clone(), 0).is_some() {
                    return Err("Ressource RAR dupliquée".into());
                }
                if entry.is_file() && is_config(&name) {
                    if configs.contains(&name) {
                        return Err("Configuration RAR dupliquée".into());
                    }
                    configs.push(name);
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
                if !entry.is_dir() {
                    let safe = safe_archive_path(&entry.name().map_err(|e| e.to_string())?)?;
                    if resources.insert(safe, index).is_some() {
                        return Err("Ressource ZIP dupliquée".into());
                    }
                }
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
            rar,
            resources: std::sync::Arc::new(resources),
            added: BTreeMap::new(),
            removed: Default::default(),
        })
    }

    /// Chemins relatifs, ordonnés avec pack_info.dynx en premier.
    pub fn configs(&self) -> &[PathBuf] {
        &self.configs
    }
    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn has_structure_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty()
    }
    pub fn add_configuration(&mut self, path: &Path, document: Document) -> Result<usize, String> {
        let path = configuration_path(path)?;
        if self
            .configs
            .iter()
            .any(|p| same_configuration_path(p, &path))
            || self
                .resources
                .keys()
                .any(|p| same_configuration_path(p, &path) && !self.removed.contains(p))
        {
            return Err("Un fichier portant ce nom existe déjà dans le pack".into());
        }
        if self.source.is_dir() && self.source.join(&path).exists() && !self.removed.contains(&path)
        {
            return Err("Un fichier ou dossier portant ce nom existe déjà dans le pack".into());
        }
        if self
            .resources
            .keys()
            .chain(self.added.keys())
            .any(|p| p.starts_with(&path) && p != &path)
        {
            return Err("Ce nom est déjà utilisé comme dossier".into());
        }
        // Aucun dossier parent ne doit être un fichier du pack.
        for parent in path
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            if self
                .resources
                .keys()
                .chain(self.added.keys())
                .any(|p| same_configuration_path(p, parent))
            {
                return Err("Un dossier parent est déjà utilisé comme fichier".into());
            }
        }
        self.removed.remove(&path);
        self.added.insert(path.clone(), document);
        self.configs.push(path);
        Ok(self.configs.len() - 1)
    }
    pub fn remove_configuration(&mut self, path: &Path) -> Result<(), String> {
        if path
            .to_string_lossy()
            .eq_ignore_ascii_case("pack_info.dynx")
        {
            return Err("Les informations du pack ne peuvent pas être supprimées".into());
        }
        let index = self
            .configs
            .iter()
            .position(|p| p == path)
            .ok_or("Configuration inconnue")?;
        self.configs.remove(index);
        if self.added.remove(path).is_none() || self.resources.contains_key(path) {
            self.removed.insert(path.to_owned());
        }
        Ok(())
    }
    pub(crate) fn materialize_edits(&self, destination: &Path) -> Result<(), String> {
        for path in &self.removed {
            match std::fs::remove_file(destination.join(path)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        for (path, document) in &self.added {
            let target = destination.join(path);
            std::fs::create_dir_all(target.parent().ok_or("Dossier invalide")?)
                .map_err(|e| e.to_string())?;
            std::fs::write(target, document.source()).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn obj_resources(&self) -> Vec<PathBuf> {
        self.resources
            .keys()
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("obj")))
            .cloned()
            .collect()
    }
    pub fn contains_resource(&self, path: &Path) -> bool {
        self.resources.contains_key(path)
    }

    /// Lit une ressource du pack avec une limite, sans extraction sur disque.
    pub fn read_resource(&self, path: &Path, maximum: u64) -> Result<Vec<u8>, String> {
        let path = safe_archive_path(&path.to_string_lossy())?;
        let index = *self
            .resources
            .get(&path)
            .ok_or_else(|| format!("Ressource absente : {}", path.display()))?;
        if self.rar {
            let mut archive = unrar::Archive::new(&self.source)
                .open_for_processing()
                .map_err(|e| e.to_string())?;
            while let Some(entry) = archive.read_header().map_err(|e| e.to_string())? {
                let name = safe_archive_path(&entry.entry().filename.to_string_lossy())?;
                if name == path {
                    if entry.entry().is_encrypted()
                        || entry.entry().is_split()
                        || entry.entry().file_attr & 0o170000 == 0o120000
                    {
                        return Err("Ressource RAR non prise en charge".into());
                    }
                    if entry.entry().unpacked_size > maximum {
                        return Err("Modèle trop volumineux".into());
                    }
                    let (bytes, _) = entry.read().map_err(|e| e.to_string())?;
                    if bytes.len() as u64 > maximum {
                        return Err("Modèle trop volumineux".into());
                    }
                    return Ok(bytes);
                }
                archive = entry.skip().map_err(|e| e.to_string())?;
            }
            return Err("L’archive a changé : rouvrir le pack".into());
        }
        if self.archive_indices.is_some() {
            let mut archive =
                zip::ZipArchive::new(File::open(&self.source).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let entry = archive.by_index(index).map_err(|e| e.to_string())?;
            if safe_archive_path(&entry.name().map_err(|e| e.to_string())?)? != path {
                return Err("L’archive a changé : rouvrir le pack".into());
            }
            if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
                return Err("Le modèle est un lien symbolique".into());
            }
            return read_resource_limited(entry, maximum);
        }
        let full = fs::canonicalize(self.source.join(&path)).map_err(|e| e.to_string())?;
        if !full.starts_with(&self.source) {
            return Err("Le modèle sort du dossier du pack".into());
        }
        read_resource_limited(File::open(full).map_err(|e| e.to_string())?, maximum)
    }

    pub fn read_config(&self, index: usize) -> Result<Document, String> {
        let path = self.configs.get(index).ok_or("Configuration inconnue")?;
        if let Some(document) = self.added.get(path) {
            return Ok(document.clone());
        }
        let source = if self.rar {
            let mut archive = unrar::Archive::new(&self.source)
                .open_for_processing()
                .map_err(|e| e.to_string())?;
            let mut result = None;
            while let Some(entry) = archive.read_header().map_err(|e| e.to_string())? {
                let name = safe_archive_path(&entry.entry().filename.to_string_lossy())?;
                if &name == path {
                    if entry.entry().unpacked_size > MAX_CONFIG_BYTES {
                        return Err("Configuration trop volumineuse (maximum 8 Mio)".into());
                    }
                    let (bytes, _) = entry.read().map_err(|e| e.to_string())?;
                    result = Some(read_limited(bytes.as_slice())?);
                    break;
                }
                archive = entry.skip().map_err(|e| e.to_string())?;
            }
            result.ok_or("Configuration absente de l'archive RAR")?
        } else if let Some(indices) = &self.archive_indices {
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

/// Valide aussi les séparateurs Windows lors d'une lecture sur Unix.
pub(crate) fn safe_archive_path(name: &str) -> Result<PathBuf, String> {
    let normalized = name.replace('\\', "/");
    let normalized = normalized.trim_end_matches('/');
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.contains(':')
        || normalized
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == ".." || part.contains('\0'))
    {
        return Err(format!("Chemin non sûr dans l'archive : {name}"));
    }
    Ok(PathBuf::from(normalized))
}

impl Pack {
    /// Copie toutes les ressources dans un dossier temporaire vide, sans
    /// laisser une bibliothèque d'archives écrire des chemins non contrôlés.
    pub(crate) fn materialize(&self, destination: &Path) -> Result<(), String> {
        if self.source.is_dir() {
            let mut pending = vec![self.source.clone()];
            while let Some(directory) = pending.pop() {
                for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let kind = entry.file_type().map_err(|e| e.to_string())?;
                    if kind.is_symlink() {
                        return Err(
                            "Export interrompu : le pack contient un lien symbolique".into()
                        );
                    }
                    let relative = entry
                        .path()
                        .strip_prefix(&self.source)
                        .map_err(|e| e.to_string())?
                        .to_owned();
                    let target = destination.join(relative);
                    if kind.is_dir() {
                        fs::create_dir_all(&target).map_err(|e| e.to_string())?;
                        pending.push(entry.path());
                    } else if kind.is_file() {
                        fs::copy(entry.path(), target).map_err(|e| e.to_string())?;
                    } else {
                        return Err("Type de ressource non pris en charge".into());
                    }
                }
            }
        } else if self.rar {
            let mut archive = unrar::Archive::new(&self.source)
                .open_for_processing()
                .map_err(|e| e.to_string())?;
            while let Some(entry) = archive.read_header().map_err(|e| e.to_string())? {
                let name = safe_archive_path(&entry.entry().filename.to_string_lossy())?;
                let target = destination.join(name);
                if entry.entry().file_attr & 0o170000 == 0o120000 {
                    return Err("Export interrompu : lien symbolique dans le RAR".into());
                }
                if entry.entry().is_directory() {
                    fs::create_dir_all(target).map_err(|e| e.to_string())?;
                    archive = entry.skip().map_err(|e| e.to_string())?;
                } else {
                    if entry.entry().unpacked_size > 512 * 1024 * 1024 {
                        return Err("Ressource RAR supérieure à 512 Mio".into());
                    }
                    let (bytes, rest) = entry.read().map_err(|e| e.to_string())?;
                    archive = rest;
                    fs::create_dir_all(target.parent().ok_or("Chemin invalide")?)
                        .map_err(|e| e.to_string())?;
                    fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(target)
                        .and_then(|mut file| {
                            use std::io::Write;
                            file.write_all(&bytes)
                        })
                        .map_err(|e| e.to_string())?;
                }
            }
        } else {
            let mut archive =
                zip::ZipArchive::new(File::open(&self.source).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            for index in 0..archive.len() {
                let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
                let name = safe_archive_path(&entry.name().map_err(|e| e.to_string())?)?;
                if entry
                    .unix_mode()
                    .is_some_and(|mode| mode & 0o170000 == 0o120000)
                {
                    return Err("Export interrompu : lien symbolique dans l'archive".into());
                }
                let target = destination.join(name);
                if entry.is_dir() {
                    fs::create_dir_all(target).map_err(|e| e.to_string())?;
                } else {
                    fs::create_dir_all(target.parent().ok_or("Chemin invalide")?)
                        .map_err(|e| e.to_string())?;
                    let mut file = fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(target)
                        .map_err(|e| e.to_string())?;
                    std::io::copy(&mut entry, &mut file).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }
}

fn read_resource_limited(reader: impl Read, maximum: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > maximum {
        return Err("Modèle trop volumineux".into());
    }
    Ok(bytes)
}

/// Chemin portable et sûr d’une nouvelle configuration.
pub fn configuration_path(path: &Path) -> Result<PathBuf, String> {
    let path = safe_archive_path(&path.to_string_lossy())?;
    if !is_config(&path) {
        return Err("Une configuration doit avoir l’extension .dynx".into());
    }
    for component in path.components() {
        crate::export::validate_filename(&component.as_os_str().to_string_lossy())?;
    }
    Ok(path)
}

fn same_configuration_path(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .replace('\\', "/")
        .eq_ignore_ascii_case(&right.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod path_tests {
    #[test]
    fn collision_checks_normalize_both_windows_separators_and_case() {
        use std::path::Path;
        assert!(super::same_configuration_path(
            Path::new("assets\\model.obj"),
            Path::new("ASSETS/model.obj")
        ));
        assert!(!super::same_configuration_path(
            Path::new("assets/model.obj"),
            Path::new("other/model.obj")
        ));
    }
}
