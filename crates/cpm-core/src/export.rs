use crate::{Document, Pack, PackInfo};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    Folder,
    Zip,
    #[default]
    Dnxpack,
}
impl ExportFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Folder => "Dossier",
            Self::Zip => ".zip",
            Self::Dnxpack => ".dnxpack",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PackMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
}
impl PackMetadata {
    pub fn from_document(document: &Document) -> Self {
        let value = |key| {
            document
                .properties()
                .iter()
                .find(|p| p.section.is_none() && p.key == key)
                .map(|p| p.value.clone())
                .unwrap_or_default()
        };
        Self {
            name: value("PackName"),
            version: value("PackVersion"),
            description: value("Description"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub destination: PathBuf,
    pub output_name: String,
    pub format: ExportFormat,
    pub replace_existing: bool,
    pub metadata: PackMetadata,
}
impl ExportOptions {
    pub fn target(&self) -> Result<PathBuf, String> {
        let name = self.output_name.trim();
        validate_filename(name)?;
        let suffix = match self.format {
            ExportFormat::Folder => "",
            ExportFormat::Zip => ".zip",
            ExportFormat::Dnxpack => ".dnxpack",
        };
        let name = if suffix.is_empty() || name.to_lowercase().ends_with(suffix) {
            name.to_owned()
        } else {
            format!("{name}{suffix}")
        };
        Ok(self.destination.join(name))
    }
}

fn validate_filename(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "\\/:*?\"<>|".contains(c))
    {
        return Err(
            "Nom de sortie invalide : saisir un nom sans chemin ni caractères spéciaux".into(),
        );
    }
    let stem = name.split('.').next().unwrap_or("").to_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.ends_with(['1', '2', '3', '4', '5', '6', '7', '8', '9']))
    {
        return Err("Nom de sortie réservé par Windows".into());
    }
    Ok(())
}

impl Pack {
    /// Exporte les ressources et toutes les configurations en mémoire. Prépare
    /// la sortie avant remplacement et restaure l'ancienne en cas d'échec.
    pub fn export(
        &self,
        options: &ExportOptions,
        documents: &BTreeMap<PathBuf, Document>,
    ) -> Result<PathBuf, String> {
        let parent =
            fs::canonicalize(&options.destination).map_err(|e| format!("Destination : {e}"))?;
        if !parent.is_dir() {
            return Err("La destination doit être un dossier".into());
        }
        let mut resolved = options.clone();
        resolved.destination = parent.clone();
        let target = resolved.target()?;
        if options.metadata.name.trim().is_empty() {
            return Err("Le nom interne du pack ne peut pas être vide".into());
        }
        if self.source().is_dir() && parent.starts_with(self.source()) {
            return Err("Choisir un dossier de destination en dehors du pack source (par exemple son dossier parent)".into());
        }
        if self.source().starts_with(&target) && self.source() != target {
            return Err(
                "La sortie ne peut pas remplacer un dossier contenant le pack source".into(),
            );
        }
        check_existing(&target, options)?;
        for path in documents.keys() {
            if !self.configs().contains(path) {
                return Err(format!(
                    "Configuration étrangère au pack : {}",
                    path.display()
                ));
            }
        }
        let stage = tempfile::Builder::new()
            .prefix(".cpm-export-")
            .tempdir_in(&parent)
            .map_err(|e| e.to_string())?;
        let content = stage.path().join("content");
        fs::create_dir(&content).map_err(|e| e.to_string())?;
        self.materialize(&content)?;
        for (path, document) in documents {
            fs::write(content.join(path), document.source()).map_err(|e| e.to_string())?;
        }
        let info_path = content.join("pack_info.dynx");
        let mut info = Document::read_from(File::open(&info_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let old_name = PackMetadata::from_document(&info).name;
        set_root(&mut info, "PackName", &options.metadata.name)?;
        set_root(&mut info, "PackVersion", &options.metadata.version)?;
        if !options.metadata.description.is_empty()
            || info
                .properties()
                .iter()
                .any(|p| p.section.is_none() && p.key == "Description")
        {
            set_root(&mut info, "Description", &options.metadata.description)?;
        }
        PackInfo::from_document(&info).map_err(|errors| {
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        fs::write(&info_path, info.source()).map_err(|e| e.to_string())?;
        if !old_name.is_empty() && old_name != options.metadata.name.trim() {
            rename_known_references(self, &content, &old_name, options.metadata.name.trim())?;
        }
        let mut validation_errors = Vec::new();
        for path in self.configs() {
            match Document::read_from(File::open(content.join(path)).map_err(|e| e.to_string())?) {
                Ok(document) => {
                    validation_errors.extend(
                        crate::configuration_issues(path, &document)
                            .into_iter()
                            .map(|issue| format!("{} : {issue}", path.display())),
                    );
                }
                Err(error) => validation_errors.push(format!("{} : {error}", path.display())),
            }
        }
        if !validation_errors.is_empty() {
            return Err(format!(
                "Export bloqué : corriger les configurations en rouge.\n{}",
                validation_errors.join("\n")
            ));
        }
        let prepared = if options.format == ExportFormat::Folder {
            content
        } else {
            let archive = stage.path().join("pack.archive");
            write_zip(&content, &archive)?;
            archive
        };
        // Revérifier après la préparation, avant toute modification de la cible.
        check_existing(&target, options)?;
        let backup = stage.path().join("previous");
        let had_previous = target.exists();
        if had_previous {
            fs::rename(&target, &backup)
                .map_err(|e| format!("Sauvegarde de la sortie existante : {e}"))?;
        }
        if let Err(error) = fs::rename(&prepared, &target) {
            if had_previous {
                if let Err(rollback) = fs::rename(&backup, &target) {
                    let recovery = stage.keep();
                    return Err(format!("Échec export : {error}. Restauration impossible : {rollback}. Ancien pack conservé dans {}", recovery.join("previous").display()));
                }
            }
            return Err(format!("Installation de l'export : {error}"));
        }
        Ok(target)
    }
}

fn check_existing(target: &Path, options: &ExportOptions) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(target) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.to_string()),
    };
    if metadata.file_type().is_symlink() {
        return Err("La sortie existante est un lien symbolique".into());
    }
    if !options.replace_existing {
        return Err(
            "La sortie existe déjà : cocher Remplacer l'existant ou choisir un autre nom".into(),
        );
    }
    if options.format == ExportFormat::Folder {
        if !metadata.is_dir() || !target.join("pack_info.dynx").is_file() {
            return Err("Le dossier à remplacer n'est pas un pack DynamX".into());
        }
    } else if !metadata.is_file() {
        return Err("La sortie existante n'est pas un fichier".into());
    }
    Ok(())
}

fn set_root(document: &mut Document, key: &str, value: &str) -> Result<(), String> {
    if value.contains(['\r', '\n']) {
        return Err(format!("{key} doit tenir sur une seule ligne"));
    }
    let indices: Vec<_> = document
        .properties()
        .iter()
        .enumerate()
        .filter(|(_, p)| p.section.is_none() && p.key == key)
        .map(|(i, _)| i)
        .collect();
    if indices.len() > 1 {
        return Err(format!("Métadonnée dupliquée : {key}"));
    }
    if let Some(index) = indices.first() {
        document
            .set_value(*index, value)
            .map_err(|e| e.to_string())?;
    } else {
        let newline = if document.source().contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let mut source = document.source().to_owned();
        if !source.is_empty() && !source.ends_with('\n') {
            source.push_str(newline);
        }
        source.push_str(&format!("{key}: {}{newline}", value.trim()));
        *document = Document::parse(&source).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn rename_known_references(
    pack: &Pack,
    content: &Path,
    old: &str,
    new: &str,
) -> Result<(), String> {
    let prefix = format!("{old}.");
    for path in pack.configs() {
        let full = content.join(path);
        let mut document = Document::read_from(File::open(&full).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Renommage, {} : {e}", path.display()))?;
        let edits: Vec<_> = document
            .properties()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if ["AttachedWheel", "DefaultEngine", "DefaultSounds"].contains(&p.key.as_str()) {
                    p.value
                        .strip_prefix(&prefix)
                        .map(|value| (i, format!("{new}.{value}")))
                } else {
                    None
                }
            })
            .collect();
        if !edits.is_empty() {
            for (index, value) in edits {
                document
                    .set_value(index, &value)
                    .map_err(|e| e.to_string())?;
            }
            fs::write(full, document.source()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn write_zip(content: &Path, destination: &Path) -> Result<(), String> {
    let file = File::create(destination).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut pending = vec![content.to_owned()];
    while let Some(directory) = pending.pop() {
        let mut entries: Vec<_> = fs::read_dir(directory)
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path.strip_prefix(content).map_err(|e| e.to_string())?;
            let name = relative
                .to_str()
                .ok_or("Nom de ressource non UTF-8")?
                .replace('\\', "/");
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                zip.add_directory(format!("{name}/"), options)
                    .map_err(|e| e.to_string())?;
                pending.push(path);
            } else {
                zip.start_file(name, options).map_err(|e| e.to_string())?;
                io::copy(&mut File::open(path).map_err(|e| e.to_string())?, &mut zip)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    zip.finish()
        .and_then(|mut file| {
            file.flush()?;
            file.sync_all()?;
            Ok(file)
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}
