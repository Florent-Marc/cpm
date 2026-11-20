//! Imports only explicitly selected files, rewrites dependencies into a private pack folder.
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Clone, Default)]
pub struct ModelFiles {
    pub obj: Option<PathBuf>,
    pub materials: Vec<PathBuf>,
    pub textures: Vec<PathBuf>,
    pub untextured: bool,
}
#[derive(Debug)]
pub struct PreparedModel {
    pub model: PathBuf,
    pub assets: BTreeMap<PathBuf, Vec<u8>>,
    pub notes: Vec<String>,
}
fn read(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| format!("{} : {e}", path.display()))?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err(format!("Fichier trop volumineux : {}", path.display()));
    }
    Ok(bytes)
}
fn base(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_lowercase()
}
fn selected(paths: &[PathBuf]) -> Result<BTreeMap<String, &PathBuf>, String> {
    let mut map = BTreeMap::new();
    for path in paths {
        let name = base(
            &path
                .file_name()
                .ok_or("Nom de fichier absent")?
                .to_string_lossy(),
        );
        if map.insert(name, path).is_some() {
            return Err(
                "Deux fichiers sélectionnés ont le même nom ; sélectionner des noms distincts"
                    .into(),
            );
        }
    }
    Ok(map)
}
impl ModelFiles {
    pub fn prepare(&self, folder: &Path) -> Result<PreparedModel, String> {
        let obj = self.obj.as_ref().ok_or("Choisir un modèle OBJ")?;
        if !obj
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("obj"))
        {
            return Err("Choisir un fichier .obj".into());
        }
        // Validate all destination components before reading selected external files.
        let folder = crate::pack::safe_archive_path(&folder.to_string_lossy())?;
        for component in folder.components() {
            crate::export::validate_filename(&component.as_os_str().to_string_lossy())?;
        }
        if self.materials.len() + self.textures.len() > 1023 {
            return Err("Trop de fichiers sélectionnés (maximum 1024)".into());
        }
        selected(&self.materials)?;
        let texture_sources = selected(&self.textures)?;
        let obj_bytes = read(obj, 32 * 1024 * 1024)?;
        let source =
            std::str::from_utf8(&obj_bytes).map_err(|_| "Le modèle OBJ doit être en UTF-8")?;
        let mut assets = BTreeMap::new();
        let mut texture_map = BTreeMap::new();
        let mut total = 0;
        if !self.untextured {
            for (index, (name, path)) in texture_sources.iter().enumerate() {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if !["png", "jpg", "jpeg", "tga", "bmp"].contains(&ext.as_str()) {
                    return Err(format!("Texture non prise en charge : {}", path.display()));
                }
                let bytes = read(path, 32 * 1024 * 1024)?;
                total += bytes.len();
                if total > 256 * 1024 * 1024 {
                    return Err("Import limité à 256 Mio".into());
                }
                let mut reader = image::ImageReader::new(std::io::Cursor::new(&bytes))
                    .with_guessed_format()
                    .map_err(|e| e.to_string())?;
                if reader.format().is_none() {
                    reader.set_format(
                        image::ImageFormat::from_extension(&ext).ok_or("Format d’image inconnu")?,
                    );
                }
                let (w, h) = reader
                    .into_dimensions()
                    .map_err(|e| format!("Texture illisible {} : {e}", path.display()))?;
                if w == 0 || h == 0 || w > 4096 || h > 4096 {
                    return Err(format!(
                        "Redimensionner {} : maximum 4096 × 4096",
                        path.display()
                    ));
                }
                let target = format!("texture_{}.{ext}", index + 1);
                texture_map.insert(name.clone(), target.clone());
                assets.insert(folder.join(target), bytes);
            }
        }
        let mut material_names = BTreeMap::<String, String>::new();
        let mut material_targets = Vec::new();
        let mut notes = Vec::new();
        if !self.untextured {
            for (index, path) in self.materials.iter().enumerate() {
                if !path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("mtl") || e.eq_ignore_ascii_case("mlt"))
                {
                    return Err("Sélectionner des fichiers MTL".into());
                }
                let bytes = read(path, 4 * 1024 * 1024)?;
                total += bytes.len();
                if total > 256 * 1024 * 1024 {
                    return Err("Import limité à 256 Mio".into());
                }
                let source =
                    std::str::from_utf8(&bytes).map_err(|_| "Le MTL doit être en UTF-8")?;
                let mut output = String::new();
                for line in source.lines() {
                    let trimmed = line.trim();
                    let (key, value) = trimmed
                        .split_once(char::is_whitespace)
                        .unwrap_or((trimmed, ""));
                    let value = value.trim();
                    if key == "newmtl"
                        && (value.is_empty()
                            || material_names
                                .insert(value.to_lowercase(), value.into())
                                .is_some())
                    {
                        return Err(format!("Nom de matériau vide ou dupliqué : {value}"));
                    }
                    if key.starts_with("map_") || ["bump", "disp", "decal"].contains(&key) {
                        if value.starts_with('-') {
                            return Err(format!("{key} contient des options non prises en charge ; exporter un MTL avec des chemins simples"));
                        }
                        // A complete filename can contain spaces; otherwise keep the optional DynamX variant tag.
                        let full = texture_map.get(&base(value));
                        let (file, tag) = if full.is_some() {
                            (value, "")
                        } else {
                            value
                                .split_once(char::is_whitespace)
                                .map(|(f, t)| (f, t.trim()))
                                .unwrap_or((value, ""))
                        };
                        let texture=texture_map.get(&base(file)).ok_or_else(||format!("Texture manquante : {file}. Sélectionner cette image, ou choisir l’import sans matériaux."))?;
                        output.push_str(&format!(
                            "{key} {texture}{}{}\n",
                            if tag.is_empty() { "" } else { " " },
                            tag
                        ));
                    } else {
                        output.push_str(line);
                        output.push('\n');
                    }
                }
                let target = format!("material_{}.mtl", index + 1);
                material_targets.push(target.clone());
                assets.insert(folder.join(target), output.into_bytes());
            }
        }
        let mut output = String::new();
        for target in &material_targets {
            output.push_str(&format!("mtllib {target}\n"));
        }
        for line in source.lines() {
            let trimmed = line.trim();
            let (key, value) = trimmed
                .split_once(char::is_whitespace)
                .unwrap_or((trimmed, ""));
            if key == "mtllib" {
                if self.materials.is_empty() && !self.untextured {
                    return Err("Le modèle référence un MTL. Sélectionner le ou les fichiers MTL, ou choisir l’import sans matériaux.".into());
                }
                continue;
            }
            if key == "usemtl" {
                if self.untextured {
                    continue;
                }
                let name = material_names
                    .get(&value.trim().to_lowercase())
                    .ok_or_else(|| {
                        format!("Matériau absent des MTL sélectionnés : {}", value.trim())
                    })?;
                output.push_str(&format!("usemtl {name}\n"));
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }
        if self.untextured {
            notes.push("Matériaux et textures ignorés à la demande".into());
        }
        let model = folder.join("model.obj");
        let bytes = output.into_bytes();
        // Validate geometry before accepting the import, without following any original mtllib paths.
        crate::ObjMesh::parse(&bytes).map_err(|e| format!("Vérifier le modèle OBJ : {e}"))?;
        total += bytes.len();
        if total > 256 * 1024 * 1024 {
            return Err("Import limité à 256 Mio".into());
        }
        assets.insert(model.clone(), bytes);
        Ok(PreparedModel {
            model,
            assets,
            notes,
        })
    }
}
