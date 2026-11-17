//! Géométrie OBJ indépendante de l’interface et du moteur graphique.
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
    pack: Option<Pack>,
    directory: PathBuf,
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
                    directory,
                });
            }
        }
        Err(format!("Modèle OBJ absent du pack : {value}"))
    }
    pub fn load(&self) -> Result<ObjMesh, String> {
        let bytes = if let Some(pack) = &self.pack {
            pack.read_resource(&self.path, MAX_OBJ_BYTES)?
        } else {
            let full = std::fs::canonicalize(self.directory.join(&self.path))
                .map_err(|e| e.to_string())?;
            if !full.starts_with(&self.directory) {
                return Err("Le modèle sort du dossier du pack".into());
            }
            let mut bytes = Vec::new();
            std::fs::File::open(full)
                .map_err(|e| e.to_string())?
                .take(MAX_OBJ_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            bytes
        };
        ObjMesh::parse(&bytes)
    }
}

pub struct ObjMesh {
    /// Triangles non indexés : position centrée/normalisée puis normale de face.
    pub vertices: Vec<[f32; 6]>,
    pub triangles: usize,
    pub objects: usize,
}
impl ObjMesh {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() as u64 > MAX_OBJ_BYTES {
            return Err("Modèle trop volumineux (maximum 32 Mio)".into());
        }
        let (models, _) = tobj::load_obj_buf(
            &mut Cursor::new(bytes),
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ignore_points: true,
                ignore_lines: true,
            },
            |_| Ok((Vec::new(), Default::default())),
        )
        .map_err(|e| format!("OBJ invalide : {e}"))?;
        let triangles: usize = models.iter().map(|m| m.mesh.indices.len() / 3).sum();
        if triangles == 0 {
            return Err("Le modèle OBJ ne contient aucune face".into());
        }
        if triangles > MAX_TRIANGLES {
            return Err("Aperçu limité à un million de triangles".into());
        }
        let mut raw = Vec::with_capacity(triangles * 3);
        for model in &models {
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
        for triangle in raw.as_chunks::<3>().0 {
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
            for p in points {
                vertices.push([
                    p[0],
                    p[1],
                    p[2],
                    n[0] / length,
                    n[1] / length,
                    n[2] / length,
                ]);
            }
        }
        if vertices.is_empty() {
            return Err("Le modèle OBJ ne contient que des faces dégénérées".into());
        }
        Ok(Self {
            triangles: vertices.len() / 3,
            vertices,
            objects: models.len(),
        })
    }
}
