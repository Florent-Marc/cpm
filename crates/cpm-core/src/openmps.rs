//! Optional OpenMPS publication snapshots. Ordinary exports do not use this module.
use crate::{
    Document, ExportFormat, ExportOptions, Pack, PackMetadata, ParameterCatalog, Severity,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
};

pub struct PreparedOpenMps {
    pub archive: PathBuf,
    pub key: String,
    pub version: String,
    pub report: PublicationReport,
    _workspace: tempfile::TempDir,
}
#[derive(Clone, Default)]
pub struct PublicationReport {
    pub moved: Vec<(PathBuf, PathBuf)>,
    pub warnings: Vec<String>,
    pub skipped: Vec<PathBuf>,
    pub files: usize,
}
pub fn validate_mps_key(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        || !value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
    {
        return Err("L’identifiant OpenMPS doit contenir 1 à 64 lettres minuscules, chiffres, tirets ou underscores, et commencer par une lettre ou un chiffre".into());
    }
    Ok(())
}
pub fn validate_mps_version(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        || value.ends_with('.')
    {
        return Err(
            "Version OpenMPS invalide : 1 à 64 lettres/chiffres, points, tirets ou underscores"
                .into(),
        );
    }
    Ok(())
}
impl Pack {
    pub fn prepare_openmps(
        &self,
        key: &str,
        version: &str,
        documents: &BTreeMap<PathBuf, Document>,
        catalog: &ParameterCatalog,
    ) -> Result<PreparedOpenMps, String> {
        validate_mps_key(key)?;
        validate_mps_version(version)?;
        let issues = self.audit(documents, catalog);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .map(|i| format!("{} / {} : {}", i.path.display(), i.parameter, i.message))
            .collect();
        if !errors.is_empty() {
            return Err(format!(
                "Corriger le pack avant publication :\n{}",
                errors.join("\n")
            ));
        }
        let workspace = tempfile::Builder::new()
            .prefix("cpm-openmps-")
            .tempdir()
            .map_err(|e| e.to_string())?;
        let info = documents
            .get(Path::new("pack_info.dynx"))
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| self.read_config(0))?;
        let metadata = PackMetadata {
            name: key.into(),
            version: version.into(),
            description: PackMetadata::from_document(&info).description,
        };
        let snapshot = self.export_with_catalog(
            &ExportOptions {
                destination: workspace.path().into(),
                output_name: "snapshot".into(),
                format: ExportFormat::Folder,
                replace_existing: false,
                metadata,
            },
            documents,
            catalog,
        )?;
        let pack = Pack::open(&snapshot)?;
        let destination = workspace.path().join("normalized");
        fs::create_dir(&destination).map_err(|e| e.to_string())?;
        let mut report = PublicationReport {
            warnings: issues
                .iter()
                .filter(|i| i.severity == Severity::Warning)
                .map(|i| format!("{} / {} : {}", i.path.display(), i.parameter, i.message))
                .collect(),
            ..Default::default()
        };
        let mut files = Vec::new();
        collect(&snapshot, &snapshot, &mut files)?;
        files.sort();
        let mut mapping = BTreeMap::new();
        let mut targets = BTreeSet::new();
        for path in &files {
            if !extension(path, "dynx") && skip(path) {
                report.skipped.push(path.clone());
                continue;
            }
            let config = extension(path, "dynx");
            if config && path.starts_with("assets") {
                return Err(format!("Déplacer cette configuration hors de assets/ pour la distribuer aux joueurs : {}",path.display()));
            }
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !config
                && ![
                    "obj",
                    "mtl",
                    "png",
                    "jpg",
                    "jpeg",
                    "tga",
                    "bmp",
                    "gif",
                    "webp",
                    "gltf",
                    "glb",
                    "bin",
                    "ogg",
                    "wav",
                    "dc",
                    "json",
                    "lang",
                    "txt",
                    "properties",
                ]
                .contains(&ext.as_str())
            {
                return Err(format!("Fichier non pris en charge pour la publication : {}. L’export classique reste disponible.",path.display()));
            }
            let target = if config || path.starts_with("assets") {
                path.clone()
            } else {
                PathBuf::from("assets").join(key).join(path)
            };
            if target.starts_with("assets") {
                let components: Vec<_> = target.components().collect();
                if components.len() < 3 {
                    return Err(format!(
                        "Ressource assets/ sans domaine : {}",
                        target.display()
                    ));
                }
                let domain = components[1].as_os_str().to_string_lossy();
                if domain.is_empty()
                    || !domain.bytes().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c)
                    })
                {
                    return Err(format!("Domaine de ressources invalide : {domain}"));
                }
            }
            let lower = target.to_string_lossy().replace('\\', "/").to_lowercase();
            if !targets.insert(lower) {
                return Err(format!(
                    "Deux ressources aboutissent au même chemin : {}",
                    target.display()
                ));
            }
            if target != *path {
                report.moved.push((path.clone(), target.clone()));
            }
            mapping.insert(path.clone(), target);
        }
        for (path, target) in &mapping {
            let full = snapshot.join(path);
            let output = destination.join(target);
            fs::create_dir_all(output.parent().ok_or("Dossier de ressource invalide")?)
                .map_err(|e| e.to_string())?;
            if extension(path, "dynx") {
                let mut doc = Document::read_from(File::open(&full).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
                let edits: Vec<_> = doc
                    .properties()
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.key == "Model" && !p.value.trim().is_empty())
                    .map(|(i, p)| {
                        let resolved = pack.resolve_asset(path, &p.value)?;
                        let target = mapping.get(&resolved).ok_or_else(|| {
                            format!("Modèle exclu de la publication : {}", resolved.display())
                        })?;
                        Ok((i, namespaced(target)?))
                    })
                    .collect::<Result<_, String>>()?;
                for (index, value) in edits {
                    doc.set_value(index, &value).map_err(|e| e.to_string())?;
                }
                fs::write(&output, doc.source()).map_err(|e| e.to_string())?;
            } else if extension(path, "obj") || extension(path, "mtl") {
                if fs::metadata(&full).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
                    return Err(format!(
                        "Fichier texte trop volumineux : {}",
                        path.display()
                    ));
                }
                let source = fs::read_to_string(&full).map_err(|e| e.to_string())?;
                let rewritten = rewrite_dependencies(path, target, &source, &mapping)?;
                fs::write(output, rewritten).map_err(|e| e.to_string())?;
            } else {
                if (extension(path, "gltf") || extension(path, "json")) && target != path {
                    report.warnings.push(format!("{} : vérifier les références internes JSON/glTF après déplacement dans assets/",path.display()));
                }
                fs::copy(full, output).map_err(|e| e.to_string())?;
            }
        }
        report.files = mapping.len();
        let normalized = Pack::open(&destination)?;
        let local_errors: Vec<_> = normalized
            .audit(&BTreeMap::new(), catalog)
            .into_iter()
            .filter(|i| i.severity == Severity::Error)
            .map(|i| format!("{} / {} : {}", i.path.display(), i.parameter, i.message))
            .collect();
        if !local_errors.is_empty() {
            return Err(local_errors.join("\n"));
        }
        let archive = workspace.path().join("publication.zip");
        crate::export::write_zip(&destination, &archive)?;
        if fs::metadata(&archive).map_err(|e| e.to_string())?.len() > 512 * 1024 * 1024 {
            return Err("Archive de publication supérieure à 512 Mio".into());
        }
        Ok(PreparedOpenMps {
            archive,
            key: key.into(),
            version: version.into(),
            report,
            _workspace: workspace,
        })
    }
}
fn collect(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err("Une ressource est un lien symbolique".into());
        }
        if kind.is_dir() {
            collect(root, &entry.path(), files)?;
        } else if kind.is_file() {
            files.push(
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_owned(),
            );
        }
    }
    Ok(())
}
fn skip(path: &Path) -> bool {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    path.components()
        .any(|c| matches!(c,Component::Normal(s) if s==".git"||s=="__MACOSX"))
        || [
            ".ds_store",
            "thumbs.db",
            "desktop.ini",
            "license",
            "licence",
        ]
        .contains(&name.as_str())
        || name.starts_with("readme")
        || name.starts_with("lisezmoi")
        || name.starts_with("mpsrepositories")
        || name.ends_with(".desc")
        || name.ends_with(".example")
        || name.ends_with(".md")
        || ((name.starts_with("client-") || name.starts_with("repo-")) && name.ends_with(".zip"))
}
fn extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .is_some_and(|v| v.eq_ignore_ascii_case(ext))
}
fn namespaced(path: &Path) -> Result<String, String> {
    let parts: Vec<_> = path.components().collect();
    if parts.len() < 3 || parts[0].as_os_str() != "assets" {
        return Err("Modèle en dehors des domaines assets/".into());
    }
    let domain = parts[1].as_os_str().to_string_lossy();
    let rest: PathBuf = parts[2..].iter().map(|c| c.as_os_str()).collect();
    Ok(format!(
        "{domain}:{}",
        rest.to_string_lossy().replace('\\', "/")
    ))
}
fn original_related(base: &Path, value: &str) -> Result<PathBuf, String> {
    let value = value.trim().trim_matches('"').replace('\\', "/");
    if let Some((domain, path)) = value.split_once(':') {
        return crate::pack::safe_archive_path(&format!("assets/{domain}/{path}"));
    }
    if value.starts_with('/') {
        return Err("Dépendance absolue non prise en charge".into());
    }
    let mut parts: Vec<String> = base
        .parent()
        .unwrap_or(Path::new(""))
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    for part in value.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop().ok_or("Une dépendance sort du pack")?;
            }
            _ => parts.push(part.into()),
        }
    }
    crate::pack::safe_archive_path(&parts.join("/"))
}
fn relative(base: &Path, target: &Path) -> String {
    let from: Vec<_> = base
        .parent()
        .unwrap_or(Path::new(""))
        .components()
        .collect();
    let to: Vec<_> = target.components().collect();
    let same = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    std::iter::repeat_n("..".to_owned(), from.len() - same)
        .chain(
            to[same..]
                .iter()
                .map(|c| c.as_os_str().to_string_lossy().into_owned()),
        )
        .collect::<Vec<_>>()
        .join("/")
}
fn rewrite_dependencies(
    old: &Path,
    new: &Path,
    source: &str,
    mapping: &BTreeMap<PathBuf, PathBuf>,
) -> Result<String, String> {
    let mut output = String::new();
    let resolve = |value: &str| -> Result<PathBuf, String> {
        let original = original_related(old, value)?;
        mapping
            .get(&original)
            .cloned()
            .ok_or_else(|| format!("{} : dépendance absente ou exclue : {value}", old.display()))
    };
    for line in source.lines() {
        let trimmed = line.trim();
        let (key, value) = trimmed
            .split_once(char::is_whitespace)
            .unwrap_or((trimmed, ""));
        let value = value.trim();
        if extension(old, "obj") && key == "mtllib" {
            let targets = if let Ok(target) = resolve(value) {
                vec![target]
            } else {
                value
                    .split_whitespace()
                    .map(resolve)
                    .collect::<Result<_, _>>()?
            };
            for target in targets {
                output.push_str(&format!("mtllib {}\n", relative(new, &target)));
            }
        } else if extension(old, "mtl")
            && (key.starts_with("map_") || ["bump", "disp", "decal"].contains(&key))
        {
            if value.starts_with('-') {
                return Err(format!(
                    "{} : options {key} non prises en charge pour la publication",
                    old.display()
                ));
            }
            if value.eq_ignore_ascii_case("white") {
                output.push_str(line);
                output.push('\n');
                continue;
            }
            let (target, tag) = if let Ok(target) = resolve(value) {
                (target, "")
            } else {
                let (path, tag) = value.split_once(char::is_whitespace).unwrap_or((value, ""));
                (resolve(path)?, tag.trim())
            };
            output.push_str(&format!(
                "{key} {}{}{}\n",
                relative(new, &target),
                if tag.is_empty() { "" } else { " " },
                tag
            ));
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    Ok(output)
}
/// Checks a downloaded players' ZIP without extracting or running anything.
pub fn validate_mps_client_archive(path: &Path, key: &str, version: &str) -> Result<(), String> {
    validate_mps_key(key)?;
    validate_mps_version(version)?;
    let mut archive = zip::ZipArchive::new(File::open(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if archive.len() > 10000 {
        return Err("Archive client trop volumineuse".into());
    }
    let mut names = BTreeSet::new();
    let mut info = false;
    let mut repositories = false;
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let name = crate::pack::safe_archive_path(&entry.name().map_err(|e| e.to_string())?)?;
        if entry.is_dir() {
            continue;
        }
        if !names.insert(name.to_string_lossy().to_lowercase()) {
            return Err("Archive client : chemin dupliqué".into());
        }
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err("Archive client : lien symbolique".into());
        }
        if name
            .components()
            .next()
            .is_some_and(|c| c.as_os_str().eq_ignore_ascii_case("assets"))
            || (!extension(&name, "dynx")
                && name != Path::new("MpsRepositories.json")
                && name != Path::new("README_INSTALLATION.txt"))
        {
            return Err(format!(
                "Le ZIP client contient une ressource qui devrait être servie par OpenMPS : {}",
                name.display()
            ));
        }
        total += entry.size();
        if entry.size() > 8 * 1024 * 1024 || total > 128 * 1024 * 1024 {
            return Err("Configurations client trop volumineuses".into());
        }
        if name == Path::new("pack_info.dynx") {
            let mut source = String::new();
            std::io::Read::by_ref(&mut entry)
                .take(8 * 1024 * 1024 + 1)
                .read_to_string(&mut source)
                .map_err(|e| e.to_string())?;
            let document = Document::parse(&source).map_err(|e| e.to_string())?;
            let meta = PackMetadata::from_document(&document);
            if meta.name != key || meta.version != version {
                return Err(
                    "Le ZIP client ne correspond pas au pack et à la version demandés".into(),
                );
            }
            info = true;
        }
        if name == Path::new("MpsRepositories.json") {
            let data: serde_json::Value = serde_json::from_reader(
                std::io::Read::by_ref(&mut entry).take(8 * 1024 * 1024 + 1),
            )
            .map_err(|e| e.to_string())?;
            if data["repositories"]
                .as_array()
                .is_none_or(|r| !r.iter().any(|r| r["key"] == key && r["version"] == version))
            {
                return Err("MpsRepositories.json ne référence pas la version attendue".into());
            }
            repositories = true;
        }
    }
    if !info || !repositories {
        return Err("Le ZIP client doit contenir pack_info.dynx et MpsRepositories.json".into());
    }
    Ok(())
}
