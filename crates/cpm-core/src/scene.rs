//! Assemblage statique des roues externes du pack ouvert.
use crate::{Document, ObjBatch, ObjMaterial, ObjMesh, ObjPart, ObjRequest, Pack};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

fn value(document: &Document, section: Option<usize>, key: &str) -> Option<String> {
    document
        .properties()
        .iter()
        .find(|p| p.section == section && p.key.eq_ignore_ascii_case(key))
        .map(|p| p.value.trim().to_owned())
}
fn vector<const N: usize>(text: &str) -> Result<[f64; N], String> {
    let values: Vec<f64> = text
        .split_whitespace()
        .map(|s| s.parse::<f64>().map_err(|_| "Nombre invalide".to_owned()))
        .collect::<Result<_, _>>()?;
    if values.len() != N || values.iter().any(|v| !v.is_finite()) {
        return Err(format!("{N} composantes numériques attendues"));
    }
    Ok(std::array::from_fn(|i| values[i]))
}
fn scale(document: &Document) -> Result<[f64; 3], String> {
    let scale = value(document, None, "ScaleModifier")
        .map(|s| vector(&s))
        .transpose()?
        .unwrap_or([1.0; 3]);
    if scale.iter().any(|v| v.abs() < 1e-12) {
        return Err(
            "ScaleModifier ne peut pas contenir une composante nulle pour cet aperçu".into(),
        );
    }
    Ok(scale)
}
fn rotate(p: [f64; 3], q: [f64; 4]) -> [f64; 3] {
    let v = [q[0], q[1], q[2]];
    let dot = v.iter().zip(p).map(|(a, b)| a * b).sum::<f64>();
    let cross = [
        v[1] * p[2] - v[2] * p[1],
        v[2] * p[0] - v[0] * p[2],
        v[0] * p[1] - v[1] * p[0],
    ];
    let vv = v.iter().map(|n| n * n).sum::<f64>();
    std::array::from_fn(|i| 2.0 * dot * v[i] + (q[3] * q[3] - vv) * p[i] + 2.0 * q[3] * cross[i])
}
fn transformed(
    v: &[f32; 8],
    basis: ([f64; 3], f64),
    scale: [f64; 3],
    vehicle_scale: [f64; 3],
    position: [f64; 3],
    right: bool,
    q: [f64; 4],
) -> Result<[f32; 8], String> {
    let mut p = std::array::from_fn(|i| (v[i] as f64 * basis.1 + basis.0[i]) * scale[i]);
    let mut n = std::array::from_fn(|i| v[i + 3] as f64 / scale[i]);
    if right {
        p[0] = -p[0];
        p[2] = -p[2];
        n[0] = -n[0];
        n[2] = -n[2];
    }
    p = rotate(p, q);
    n = rotate(n, q);
    for i in 0..3 {
        p[i] = (p[i] + position[i]) * vehicle_scale[i];
        n[i] /= vehicle_scale[i];
    }
    let length = n.iter().map(|v| v * v).sum::<f64>().sqrt();
    let result = [
        p[0] as f32,
        p[1] as f32,
        p[2] as f32,
        (n[0] / length) as f32,
        (n[1] / length) as f32,
        (n[2] / length) as f32,
        v[6],
        v[7],
    ];
    if result.iter().any(|v| !v.is_finite()) {
        return Err("Transformation du modèle hors limites".into());
    }
    Ok(result)
}
fn read_document(
    pack: &Pack,
    overrides: &BTreeMap<PathBuf, Document>,
    index: usize,
) -> Result<Document, String> {
    overrides
        .get(&pack.configs()[index])
        .cloned()
        .map(Ok)
        .unwrap_or_else(|| pack.read_config(index))
}

pub(crate) fn assemble(request: &ObjRequest) -> Result<ObjMesh, String> {
    let mut scene = request.load_geometry()?;
    let wheels: Vec<_> = request
        .document
        .sections()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.parent.is_none() && s.name.to_ascii_lowercase().starts_with("wheel"))
        .map(|(i, s)| (i, s.name.clone()))
        .collect();
    if wheels.is_empty() {
        return Ok(scene);
    }
    let pack = request
        .pack
        .clone()
        .or_else(|| Pack::open(&request.directory).ok());
    let Some(pack) = pack else {
        scene.warnings.push(
            "Roues non ajoutées : ouvrir le pack complet pour retrouver leurs configurations"
                .into(),
        );
        return Ok(scene);
    };
    let metadata = read_document(&pack, &request.overrides, 0)?;
    let pack_name = value(&metadata, None, "PackName").unwrap_or_default();
    let vehicle_scale = scale(&request.document)?;
    let body_basis = (scene.center, scene.radius);
    for vertex in &mut scene.vertices {
        *vertex = transformed(
            vertex,
            body_basis,
            [1.0; 3],
            vehicle_scale,
            [0.0; 3],
            false,
            [0.0, 0.0, 0.0, 1.0],
        )?;
    }
    let mut cache: BTreeMap<PathBuf, Arc<ObjMesh>> = BTreeMap::new();
    for (section, name) in wheels.iter().take(64) {
        let result = (|| -> Result<(), String> {
            if value(&request.document, Some(*section), "Rim").is_some_and(|s| !s.is_empty()) {
                return Err("roue intégrée au modèle ; aucune roue externe ajoutée".into());
            }
            let reference = value(&request.document, Some(*section), "AttachedWheel")
                .ok_or("AttachedWheel absent")?;
            let (namespace, filename) = reference
                .rsplit_once('.')
                .map(|(p, n)| (Some(p), n))
                .unwrap_or((None, reference.as_str()));
            if namespace.is_some_and(|p| !p.eq_ignore_ascii_case(&pack_name)) {
                return Err(format!(
                    "roue du pack externe {} indisponible",
                    namespace.unwrap()
                ));
            }
            let matches: Vec<_> = pack
                .configs()
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.file_stem()
                        .is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case(filename))
                })
                .map(|(i, _)| i)
                .collect();
            if matches.len() != 1 {
                return Err(format!(
                    "référence de roue absente ou ambiguë : {reference}"
                ));
            }
            let index = matches[0];
            let path = &pack.configs()[index];
            let document = read_document(&pack, &request.overrides, index)?;
            let source = if let Some(mesh) = cache.get(path) {
                mesh.clone()
            } else {
                Arc::new(ObjRequest::resolve(Some(&pack), path, &document)?.load_geometry()?)
            };
            let wheel_scale = scale(&document)?;
            let position = vector::<3>(
                &value(&request.document, Some(*section), "Position").ok_or("Position absente")?,
            )?;
            // VECTOR3F_INVERSED_Y dans le chargeur DynamX : x z -y.
            let position = [position[0], position[2], -position[1]];
            let right = value(&request.document, Some(*section), "IsRight")
                .map(|s| s.eq_ignore_ascii_case("true"))
                .unwrap_or_else(|| name.to_ascii_lowercase().contains("right"));
            let mut q = value(&request.document, Some(*section), "SuspensionAxis")
                .map(|s| vector::<4>(&s))
                .transpose()?
                .unwrap_or([0.0, 0.0, 0.0, 1.0]);
            let length = q.iter().map(|v| v * v).sum::<f64>().sqrt();
            if length < 1e-12 {
                return Err("SuspensionAxis invalide".into());
            }
            for v in &mut q {
                *v /= length;
            }
            let vertices: Vec<_> = source
                .vertices
                .iter()
                .map(|v| {
                    transformed(
                        v,
                        (source.center, source.radius),
                        wheel_scale,
                        vehicle_scale,
                        position,
                        right,
                        q,
                    )
                })
                .collect::<Result<_, _>>()?;
            if scene.vertices.len() + vertices.len() > 3_000_000 {
                return Err("limite totale d’un million de triangles atteinte".into());
            }
            let new_bytes: usize = source
                .textures
                .iter()
                .filter(|t| !scene.textures.iter().any(|s| s.path == t.path))
                .map(|t| t.rgba.len())
                .sum();
            if scene.textures.iter().map(|t| t.rgba.len()).sum::<usize>() + new_bytes
                > 256 * 1024 * 1024
            {
                return Err("limite totale de textures atteinte".into());
            }
            let texture_map: Vec<_> = source
                .textures
                .iter()
                .map(|texture| {
                    scene
                        .textures
                        .iter()
                        .position(|t| t.path == texture.path)
                        .unwrap_or_else(|| {
                            scene.textures.push(texture.clone());
                            scene.textures.len() - 1
                        })
                })
                .collect();
            // Bound retained source geometry as well as the assembled scene.
            if !cache.contains_key(path)
                && cache.values().map(|m| m.vertices.len()).sum::<usize>() + source.vertices.len()
                    <= 3_000_000
                && cache
                    .values()
                    .flat_map(|m| &m.textures)
                    .map(|t| t.rgba.len())
                    .sum::<usize>()
                    + source.textures.iter().map(|t| t.rgba.len()).sum::<usize>()
                    <= 256 * 1024 * 1024
            {
                cache.insert(path.clone(), source.clone());
            }
            let material_offset = scene.materials.len();
            scene
                .materials
                .extend(source.materials.iter().map(|m| ObjMaterial {
                    name: format!("{name}/{}", m.name),
                    diffuse: m.diffuse,
                    texture: m.texture.map(|i| texture_map[i]),
                }));
            let offset = scene.vertices.len();
            scene.vertices.extend(vertices);
            for part in &source.parts {
                scene.parts.push(ObjPart {
                    name: format!("{name}/{}", part.name),
                    batches: part
                        .batches
                        .iter()
                        .map(|b| ObjBatch {
                            first: b.first + offset,
                            count: b.count,
                            material: b.material.map(|i| i + material_offset),
                            has_uv: b.has_uv,
                        })
                        .collect(),
                });
            }
            scene
                .warnings
                .extend(source.warnings.iter().map(|w| format!("{name} : {w}")));
            if ["DependsOn", "MudGuard"].iter().any(|key| {
                value(&request.document, Some(*section), key).is_some_and(|v| !v.is_empty())
            }) {
                scene.warnings.push(format!(
                    "{name} : les dépendances et garde-boue articulés ne sont pas simulés"
                ));
            }
            Ok(())
        })();
        if let Err(error) = result {
            scene.warnings.push(format!("{name} : {error}"));
        }
    }
    if wheels.len() > 64 {
        scene
            .warnings
            .push("L’aperçu est limité aux 64 premières roues".into());
    }
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for vertex in &scene.vertices {
        for i in 0..3 {
            min[i] = min[i].min(vertex[i] as f64);
            max[i] = max[i].max(vertex[i] as f64);
        }
    }
    let center = std::array::from_fn::<_, 3, _>(|i| (min[i] + max[i]) * 0.5);
    let radius = scene
        .vertices
        .iter()
        .map(|v| {
            (0..3)
                .map(|i| (v[i] as f64 - center[i]).powi(2))
                .sum::<f64>()
                .sqrt()
        })
        .fold(0.0, f64::max);
    if radius == 0.0 || !radius.is_finite() {
        return Err("Assemblage 3D de taille invalide".into());
    }
    for vertex in &mut scene.vertices {
        for i in 0..3 {
            vertex[i] = ((vertex[i] as f64 - center[i]) / radius) as f32;
        }
    }
    scene.center = center;
    scene.radius = radius;
    scene.objects = scene.parts.len();
    scene.triangles = scene.vertices.len() / 3;
    scene.warnings.sort();
    scene.warnings.dedup();
    Ok(scene)
}
