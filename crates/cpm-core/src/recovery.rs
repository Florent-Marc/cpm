//! Recover into a fresh private directory; never overwrite an existing local pack.
use crate::Pack;
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub fn recover_pack_archive(
    archive: &Path,
    parent: &Path,
    key: &str,
    version: &str,
) -> Result<PathBuf, String> {
    crate::validate_mps_key(key)?;
    crate::validate_mps_version(version)?;
    let parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
    let folder = tempfile::Builder::new()
        .prefix(&format!("{key}-{version}-"))
        .tempdir_in(parent)
        .map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(fs::File::open(archive).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if zip.len() > 10000 {
        return Err("Archive de récupération trop volumineuse".into());
    }
    let mut seen = BTreeSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
        let path = crate::pack::safe_archive_path(&file.name().map_err(|e| e.to_string())?)?;
        for component in path.components() {
            crate::export::validate_filename(&component.as_os_str().to_string_lossy())?;
        }
        if !seen.insert(path.to_string_lossy().to_lowercase()) {
            return Err("Chemin dupliqué dans la récupération".into());
        }
        if file.unix_mode().is_some_and(|m| {
            matches!(
                m & 0o170000,
                0o120000 | 0o060000 | 0o020000 | 0o010000 | 0o140000
            )
        }) {
            return Err("Lien ou fichier spécial dans la récupération".into());
        }
        if file.is_dir() {
            continue;
        }
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ![
            "dynx",
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
            "md",
        ]
        .contains(&extension.as_str())
            && !path.file_name().is_some_and(|n| {
                n.eq_ignore_ascii_case("license") || n.eq_ignore_ascii_case("readme")
            })
        {
            return Err("Fichier non pris en charge dans la copie éditable".into());
        }
        total = total
            .checked_add(file.size())
            .ok_or("Archive trop volumineuse")?;
        let limit = if extension == "dynx" {
            8 * 1024 * 1024
        } else {
            128 * 1024 * 1024
        };
        if file.size() > limit || total > 1024 * 1024 * 1024 {
            return Err("Limite de récupération dépassée".into());
        }
        let target = folder.path().join(&path);
        fs::create_dir_all(target.parent().ok_or("Chemin invalide")?).map_err(|e| e.to_string())?;
        let size = file.size();
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)
            .map_err(|e| e.to_string())?;
        let count = std::io::copy(&mut Read::by_ref(&mut file).take(limit + 1), &mut output)
            .map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
        if count != size {
            return Err("Taille de fichier incohérente".into());
        }
    }
    let pack = Pack::open(folder.path())?;
    for i in 0..pack.configs().len() {
        pack.read_config(i)?;
    }
    let info = pack.read_config(0)?;
    if crate::PackMetadata::from_document(&info)
        .name
        .trim()
        .is_empty()
    {
        return Err("PackName absent de la copie éditable".into());
    }
    Ok(folder.keep())
}
