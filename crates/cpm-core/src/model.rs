//! Géométrie OBJ, pièces nommées et matériaux texturés indépendants de l’interface.
use crate::{Document, Pack};
use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
const MAX_OBJ_BYTES: u64 = 32 * 1024 * 1024;
const MAX_TRIANGLES: usize = 1_000_000;

#[derive(Clone)]
pub struct ObjRequest {
    pub path: PathBuf,
    pub(crate) pack: Option<Pack>,
    pub(crate) document: Document,
    pub(crate) overrides: std::collections::BTreeMap<PathBuf, Document>,
    pub(crate) directory: PathBuf,
}
impl ObjRequest {
    /// Résout le Model racine, jamais un modèle arbitraire dans un autre pack.
    pub fn resolve(
        pack: Option<&Pack>,
        config: &Path,
        document: &Document,
    ) -> Result<Self, String> {
        let value = document
            .properties()
            .iter()
            .find(|p| p.section.is_none() && p.key == "Model")
            .ok_or("Cette configuration ne déclare pas de Model")?
            .value
            .trim();
        let normalized = value.replace('\\', "/");
        let directory = if let Some(pack) = pack {
            pack.source().to_owned()
        } else {
            let parent = config
                .parent()
                .ok_or("Dossier de configuration introuvable")?;
            let root = parent
                .ancestors()
                .find(|p| p.join("pack_info.dynx").is_file())
                .unwrap_or(parent);
            std::fs::canonicalize(root).map_err(|e| e.to_string())?
        };
        let mut candidates = Vec::new();
        if let Some((namespace, relative)) = normalized.split_once(':') {
            let namespace = crate::pack::safe_archive_path(namespace)?;
            if namespace.components().count() != 1 {
                return Err("Espace de noms de modèle invalide".into());
            }
            let relative = crate::pack::safe_archive_path(relative)?;
            candidates.push(PathBuf::from("assets").join(&namespace).join(&relative));
            candidates.push(
                PathBuf::from("assets")
                    .join(&namespace)
                    .join("models")
                    .join(&relative),
            );
        } else {
            let relative = crate::pack::safe_archive_path(&normalized)?;
            candidates.push(relative.clone());
            candidates.push(PathBuf::from("models").join(&relative));
            for namespace in ["dynamxmod", "minecraft"] {
                candidates.push(PathBuf::from("assets").join(namespace).join(&relative));
                candidates.push(
                    PathBuf::from("assets")
                        .join(namespace)
                        .join("models")
                        .join(&relative),
                );
            }
            if let Some(parent) = config.parent() {
                let parent = if pack.is_some() {
                    parent.to_owned()
                } else {
                    parent
                        .strip_prefix(&directory)
                        .unwrap_or(Path::new(""))
                        .to_owned()
                };
                candidates.push(parent.join(&relative));
            }
        }
        if !candidates[0]
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("obj"))
        {
            return Err("L’aperçu prend en charge les modèles .obj".into());
        }
        for path in candidates {
            let exists = if let Some(pack) = pack {
                pack.contains_resource(&path)
            } else {
                std::fs::canonicalize(directory.join(&path))
                    .is_ok_and(|p| p.starts_with(&directory) && p.is_file())
            };
            if exists {
                return Ok(Self {
                    path,
                    pack: pack.cloned(),
                    document: document.clone(),
                    overrides: Default::default(),
                    directory,
                });
            }
        }
        Err(format!("Modèle OBJ absent du pack : {value}"))
    }
    fn read(&self, path: &Path, limit: u64) -> Result<Vec<u8>, String> {
        if let Some(pack) = &self.pack {
            return pack.read_resource(path, limit);
        }
        let full = std::fs::canonicalize(self.directory.join(path)).map_err(|e| e.to_string())?;
        if !full.starts_with(&self.directory) {
            return Err("La ressource sort du dossier du pack".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(full)
            .map_err(|e| e.to_string())?
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > limit {
            return Err("Ressource trop volumineuse".into());
        }
        Ok(bytes)
    }
    pub fn with_documents(
        mut self,
        documents: std::collections::BTreeMap<PathBuf, Document>,
    ) -> Self {
        self.overrides = documents;
        self
    }
    pub fn load(&self) -> Result<ObjMesh, String> {
        crate::scene::assemble(self)
    }
    pub(crate) fn load_geometry(&self) -> Result<ObjMesh, String> {
        ObjMesh::parse_with(&self.read(&self.path, MAX_OBJ_BYTES)?, Some(self))
    }
}

/// Chemin relatif au fichier qui le référence, en autorisant ../ à l’intérieur du pack.
fn related_path(base: &Path, value: &str) -> Result<PathBuf, String> {
    let value = value.trim().trim_matches('"').replace('\\', "/");
    if value.is_empty() || value.starts_with('/') || value.contains('\0') {
        return Err("Chemin de ressource invalide".into());
    }
    if let Some((namespace, path)) = value.split_once(':') {
        let namespace = crate::pack::safe_archive_path(namespace)?;
        if namespace.components().count() != 1 {
            return Err("Espace de noms invalide".into());
        }
        return Ok(PathBuf::from("assets")
            .join(namespace)
            .join(crate::pack::safe_archive_path(path)?));
    }
    let mut path = base.parent().unwrap_or(Path::new("")).to_owned();
    for part in value.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if !path.pop() {
                    return Err("La ressource sort du dossier du pack".into());
                }
            }
            _ => path.push(part),
        }
    }
    crate::pack::safe_archive_path(&path.to_string_lossy())
}

#[derive(Clone)]
pub struct ObjBatch {
    pub first: usize,
    pub count: usize,
    pub material: Option<usize>,
    pub has_uv: bool,
}
#[derive(Clone)]
pub struct ObjPart {
    pub name: String,
    pub batches: Vec<ObjBatch>,
}
#[derive(Clone)]
pub struct ObjMaterial {
    pub name: String,
    pub diffuse: [f32; 4],
    pub texture: Option<usize>,
}
#[derive(Clone)]
pub struct ObjTexture {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub has_alpha: bool,
}

#[derive(Clone)]
pub struct ObjMesh {
    pub(crate) center: [f64; 3],
    pub(crate) radius: f64,
    /// Triangles non indexés : position centrée/normalisée, normale de face puis UV.
    pub vertices: Vec<[f32; 8]>,
    pub parts: Vec<ObjPart>,
    pub materials: Vec<ObjMaterial>,
    pub textures: Vec<ObjTexture>,
    pub warnings: Vec<String>,
    pub triangles: usize,
    pub objects: usize,
}
impl ObjMesh {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        Self::parse_with(bytes, None)
    }
    fn parse_with(bytes: &[u8], request: Option<&ObjRequest>) -> Result<Self, String> {
        if bytes.len() as u64 > MAX_OBJ_BYTES {
            return Err("Modèle trop volumineux (maximum 32 Mio)".into());
        }
        let warnings = std::cell::RefCell::new(Vec::new());
        let mtl_budget = std::cell::Cell::new(0usize);
        let (models, materials) = tobj::load_obj_buf(
            &mut Cursor::new(bytes),
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ignore_points: true,
                ignore_lines: true,
            },
            |path| {
                let loaded = (|| -> Result<_, String> {
                    let request =
                        request.ok_or("Lecture des matériaux indisponible sans dossier de pack")?;
                    let path = related_path(&request.path, &path.to_string_lossy())?;
                    let bytes = request.read(&path, 4 * 1024 * 1024)?;
                    let used = mtl_budget.get() + bytes.len();
                    if used > 16 * 1024 * 1024 {
                        return Err("Limite de matériaux atteinte (16 Mio)".into());
                    }
                    mtl_budget.set(used);
                    let (mut materials, map) =
                        tobj::load_mtl_buf(&mut Cursor::new(bytes)).map_err(|e| e.to_string())?;
                    for material in &mut materials {
                        if let Some(texture) = &material.diffuse_texture {
                            let resolved = if texture.trim().starts_with('-') {
                                Err("Options map_Kd non prises en charge".into())
                            } else {
                                related_path(&path, texture)
                            };
                            match resolved {
                                Ok(path) => {
                                    material.diffuse_texture =
                                        Some(path.to_string_lossy().into_owned())
                                }
                                Err(error) => {
                                    warnings
                                        .borrow_mut()
                                        .push(format!("{} : {error}", material.name));
                                    material.diffuse_texture = None;
                                }
                            }
                        }
                    }
                    Ok((materials, map))
                })();
                match loaded {
                    Ok(value) => Ok(value),
                    Err(error) => {
                        warnings
                            .borrow_mut()
                            .push(format!("Matériau {} : {error}", path.display()));
                        Ok((Vec::new(), Default::default()))
                    }
                }
            },
        )
        .map_err(|e| format!("OBJ invalide : {e}"))?;
        let mut warnings = warnings.into_inner();
        let mut textures = Vec::new();
        let mut texture_indices = std::collections::BTreeMap::new();
        let mut texture_bytes = 0usize;
        let mut converted_materials = Vec::new();
        for material in materials.map_err(|e| e.to_string())? {
            let mut texture = None;
            if let (Some(path), Some(request)) = (&material.diffuse_texture, request) {
                let path = PathBuf::from(path);
                texture = *texture_indices.entry(path.clone()).or_insert_with(|| {
                    match request
                        .read(&path, 32 * 1024 * 1024)
                        .and_then(|bytes| decode_texture(&path, bytes))
                    {
                        Ok(image) if texture_bytes + image.rgba.len() <= 256 * 1024 * 1024 => {
                            texture_bytes += image.rgba.len();
                            let index = textures.len();
                            textures.push(image);
                            Some(index)
                        }
                        Ok(_) => {
                            warnings.push("Limite totale de textures atteinte (256 Mio)".into());
                            None
                        }
                        Err(error) => {
                            warnings.push(format!("Texture {} : {error}", path.display()));
                            None
                        }
                    }
                });
            }
            let color = material.diffuse.unwrap_or(if texture.is_some() {
                [1.0; 3]
            } else {
                [0.56, 0.68, 0.82]
            });
            let color = color.map(|v| {
                if v.is_finite() {
                    v.clamp(0.0, 1.0)
                } else {
                    1.0
                }
            });
            let alpha = material
                .dissolve
                .filter(|v| v.is_finite())
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            converted_materials.push(ObjMaterial {
                name: material.name,
                diffuse: [color[0], color[1], color[2], alpha],
                texture,
            });
        }
        let triangles: usize = models.iter().map(|m| m.mesh.indices.len() / 3).sum();
        if triangles == 0 {
            return Err("Le modèle OBJ ne contient aucune face".into());
        }
        if triangles > MAX_TRIANGLES {
            return Err("Aperçu limité à un million de triangles".into());
        }
        let mut raw = Vec::with_capacity(triangles * 3);
        let mut attributes = Vec::with_capacity(triangles * 3);
        for (model_index, model) in models.iter().enumerate() {
            for triangle in model.mesh.indices.as_chunks::<3>().0 {
                for index in triangle {
                    let start = *index as usize * 3;
                    let p = model
                        .mesh
                        .positions
                        .get(start..start + 3)
                        .ok_or("Indice OBJ invalide")?;
                    if !p.iter().all(|v| v.is_finite()) {
                        return Err("Le modèle contient des coordonnées invalides".into());
                    }
                    raw.push([p[0] as f64, p[1] as f64, p[2] as f64]);
                    let uv = model
                        .mesh
                        .texcoords
                        .get(*index as usize * 2..*index as usize * 2 + 2);
                    let valid = uv.is_some_and(|uv| uv.iter().all(|v| v.is_finite()));
                    attributes.push((
                        model_index,
                        if valid {
                            [uv.unwrap()[0], 1.0 - uv.unwrap()[1]]
                        } else {
                            [0.0; 2]
                        },
                        valid,
                    ));
                }
            }
        }
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for p in &raw {
            for axis in 0..3 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        }
        let center = std::array::from_fn::<_, 3, _>(|i| (min[i] + max[i]) * 0.5);
        let radius = raw
            .iter()
            .map(|p| {
                (0..3)
                    .map(|i| (p[i] - center[i]).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .fold(0.0, f64::max);
        if radius == 0.0 {
            return Err("Le modèle OBJ a une taille nulle".into());
        }
        let mut vertices = Vec::with_capacity(raw.len());
        let mut parts: Vec<ObjPart> = Vec::new();
        let mut part_indices = std::collections::BTreeMap::new();
        for (triangle, attrs) in raw
            .as_chunks::<3>()
            .0
            .iter()
            .zip(attributes.as_chunks::<3>().0)
        {
            let points: Vec<[f32; 3]> = triangle
                .iter()
                .map(|p| std::array::from_fn(|i| ((p[i] - center[i]) / radius) as f32))
                .collect();
            let a: [f32; 3] = std::array::from_fn(|i| points[1][i] - points[0][i]);
            let b: [f32; 3] = std::array::from_fn(|i| points[2][i] - points[0][i]);
            let n = [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ];
            let length = n.iter().map(|v| v * v).sum::<f32>().sqrt();
            if length == 0.0 {
                continue;
            }
            let model = &models[attrs[0].0];
            let part_index = *part_indices.entry(model.name.clone()).or_insert_with(|| {
                parts.push(ObjPart {
                    name: model.name.clone(),
                    batches: Vec::new(),
                });
                parts.len() - 1
            });
            let part = &mut parts[part_index];
            let has_uv = attrs.iter().all(|a| a.2);
            if let Some(batch) = part.batches.last_mut().filter(|b| {
                b.first + b.count == vertices.len()
                    && b.material == model.mesh.material_id
                    && b.has_uv == has_uv
            }) {
                batch.count += 3;
            } else {
                part.batches.push(ObjBatch {
                    first: vertices.len(),
                    count: 3,
                    material: model.mesh.material_id,
                    has_uv,
                });
            }
            for (p, attr) in points.into_iter().zip(attrs) {
                vertices.push([
                    p[0],
                    p[1],
                    p[2],
                    n[0] / length,
                    n[1] / length,
                    n[2] / length,
                    attr.1[0],
                    attr.1[1],
                ]);
            }
        }
        if vertices.is_empty() {
            return Err("Le modèle OBJ ne contient que des faces dégénérées".into());
        }
        for part in &parts {
            if part.batches.iter().any(|b| {
                !b.has_uv
                    && b.material
                        .and_then(|i| converted_materials.get(i))
                        .is_some_and(|m| m.texture.is_some())
            }) {
                warnings.push(format!(
                    "{} : coordonnées UV absentes ; couleur du matériau utilisée",
                    part.name
                ));
            }
        }
        warnings.sort();
        warnings.dedup();
        Ok(Self {
            center,
            radius,
            objects: parts.len(),
            parts,
            materials: converted_materials,
            textures,
            warnings,
            triangles: vertices.len() / 3,
            vertices,
        })
    }
}

fn decode_texture(path: &Path, bytes: Vec<u8>) -> Result<ObjTexture, String> {
    let format = image::guess_format(&bytes)
        .ok()
        .or_else(|| image::ImageFormat::from_path(path).ok())
        .ok_or("Format d’image inconnu")?;
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(96 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().map_err(|e| e.to_string())?.into_rgba8();
    let has_alpha = image.pixels().any(|p| p.0[3] < 255);
    Ok(ObjTexture {
        path: path.to_owned(),
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
        has_alpha,
    })
}
