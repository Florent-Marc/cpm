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
    let pack = request
        .pack
        .clone()
        .or_else(|| Pack::open(&request.directory).ok());
    let pack_name = pack
        .as_ref()
        .and_then(|p| read_document(p, &request.overrides, 0).ok())
        .and_then(|d| value(&d, None, "PackName"))
        .unwrap_or_default();
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
    steering(&mut scene, &request.document, vehicle_scale);
    collision_boxes(&mut scene, &request.document, vehicle_scale);
    let mut cache: BTreeMap<PathBuf, Arc<ObjMesh>> = BTreeMap::new();
    for (section, name) in wheels.iter().take(64) {
        let result = (|| -> Result<(), String> {
            let pack = pack
                .as_ref()
                .ok_or("ouvrir le pack complet pour retrouver les roues")?;
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
            let document = read_document(pack, &request.overrides, index)?;
            let source = if let Some(mesh) = cache.get(path) {
                mesh.clone()
            } else {
                Arc::new(
                    ObjRequest::resolve(Some(pack), path, &document)?
                        .with_variant(request.variant)
                        .load_geometry()?,
                )
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
            let q = value(&request.document, Some(*section), "SuspensionAxis")
                .map(|s| quaternion(&s))
                .transpose()?
                .unwrap_or([0.0, 0.0, 0.0, 1.0]);
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
                    section: Some(*section),
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
    for vertex in scene.vertices.iter().map(|v| [v[0], v[1], v[2]]).chain(
        scene
            .collisions
            .iter()
            .flat_map(|c| c.vertices.iter().copied()),
    ) {
        for i in 0..3 {
            min[i] = min[i].min(vertex[i] as f64);
            max[i] = max[i].max(vertex[i] as f64);
        }
    }
    let center = std::array::from_fn::<_, 3, _>(|i| (min[i] + max[i]) * 0.5);
    let radius = scene
        .vertices
        .iter()
        .map(|v| [v[0], v[1], v[2]])
        .chain(
            scene
                .collisions
                .iter()
                .flat_map(|c| c.vertices.iter().copied()),
        )
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
    for vertex in scene.collisions.iter_mut().flat_map(|c| &mut c.vertices) {
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

// DynamX reads quaternion configuration as w x y z; rotate() uses x y z w.
fn quaternion(text: &str) -> Result<[f64; 4], String> {
    let q = vector::<4>(text)?;
    let length = q.iter().map(|n| n * n).sum::<f64>().sqrt();
    if length < 1e-12 || !length.is_finite() {
        return Err("Quaternion invalide".into());
    }
    Ok([q[1] / length, q[2] / length, q[3] / length, q[0] / length])
}
fn alias(document: &Document, section: usize, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|key| value(document, Some(section), key))
}
fn steering(scene: &mut ObjMesh, document: &Document, scale: [f64; 3]) {
    let mut used = std::collections::BTreeSet::new();
    for (section, info) in document.sections().iter().enumerate().filter(|(_, s)| {
        s.parent.is_none() && s.name.to_ascii_lowercase().starts_with("steeringwheel")
    }) {
        let result = (|| -> Result<(), String> {
            let object = alias(document, section, &["ObjectName", "PartName"])
                .unwrap_or("SteeringWheel".into());
            let matches: Vec<_> = scene
                .parts
                .iter()
                .enumerate()
                .filter(|(_, p)| p.name.eq_ignore_ascii_case(&object))
                .map(|(i, _)| i)
                .collect();
            if matches.len() != 1 {
                return Err(format!("pièce OBJ absente ou ambiguë : {object}"));
            }
            let part = matches[0];
            if !used.insert(part) {
                return Err(format!("pièce déjà affectée à un volant : {object}"));
            }
            let position = alias(document, section, &["Position", "ShapePosition"]);
            let rotation = alias(
                document,
                section,
                &["Rotation", "BaseRotation", "BaseRotationQuat"],
            )
            .map(|s| quaternion(&s))
            .transpose()?
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
            let automatic = position.is_none();
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            for batch in &scene.parts[part].batches {
                for old in &scene.vertices[batch.first..batch.first + batch.count] {
                    for i in 0..3 {
                        let raw = old[i] as f64 / scale[i];
                        min[i] = min[i].min(raw);
                        max[i] = max[i].max(raw);
                    }
                }
            }
            let pivot = std::array::from_fn::<_, 3, _>(|i| (min[i] + max[i]) * 0.5);
            let offset = if let Some(position) = position {
                let p = vector::<3>(&position)?;
                [p[0] * scale[0], p[2] * scale[1], -p[1] * scale[2]]
            } else {
                std::array::from_fn(|i| pivot[i] * scale[i])
            };
            let inverse = [-rotation[0], -rotation[1], -rotation[2], rotation[3]];
            let mut changes = Vec::new();
            for batch in &scene.parts[part].batches {
                for (index, old) in scene
                    .vertices
                    .iter()
                    .enumerate()
                    .skip(batch.first)
                    .take(batch.count)
                {
                    let old = *old;
                    let (p, n) = if automatic {
                        let p = rotate(
                            std::array::from_fn(|i| old[i] as f64 / scale[i] - pivot[i]),
                            inverse,
                        );
                        let n = rotate(
                            std::array::from_fn(|i| old[i + 3] as f64 * scale[i]),
                            inverse,
                        );
                        (
                            rotate(std::array::from_fn(|i| p[i] * scale[i]), rotation),
                            rotate(std::array::from_fn(|i| n[i] / scale[i]), rotation),
                        )
                    } else {
                        (
                            rotate([old[0] as f64, old[1] as f64, old[2] as f64], rotation),
                            rotate([old[3] as f64, old[4] as f64, old[5] as f64], rotation),
                        )
                    };
                    let length = n.iter().map(|v| v * v).sum::<f64>().sqrt();
                    let mut v = old;
                    for i in 0..3 {
                        v[i] = (p[i] + offset[i]) as f32;
                        v[i + 3] = (n[i] / length) as f32;
                    }
                    if v.iter().any(|v| !v.is_finite()) {
                        return Err("Transformation du volant hors limites".into());
                    }
                    changes.push((index, v));
                }
            }
            scene.parts[part].section = Some(section);
            for (index, v) in changes {
                scene.vertices[index] = v;
            }
            Ok(())
        })();
        if let Err(error) = result {
            scene.warnings.push(format!("{} : {error}", info.name));
        }
    }
}
fn collision_boxes(scene: &mut ObjMesh, document: &Document, scale: [f64; 3]) {
    let shapes: Vec<_> = document
        .sections()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.parent.is_none() && s.name.to_ascii_lowercase().starts_with("shape"))
        .collect();
    if shapes.len() > 1024 {
        scene
            .warnings
            .push("Affichage limité aux 1024 premières boîtes de collision".into());
    }
    for (section, info) in shapes.into_iter().take(1024) {
        let result = (|| -> Result<crate::CollisionWire, String> {
            let kind = value(document, Some(section), "Type").unwrap_or("BOX".into());
            if !kind.eq_ignore_ascii_case("box") {
                return Err(format!(
                    "forme {kind} non affichée ; seules les boîtes BOX sont prises en charge"
                ));
            }
            let p = vector::<3>(
                &alias(document, section, &["Position", "ShapePosition"])
                    .ok_or("Position absente")?,
            )?;
            let size = vector::<3>(
                &alias(
                    document,
                    section,
                    &["Scale", "Size", "ShapeScale", "BoxDim"],
                )
                .unwrap_or("0.5 1 0.5".into()),
            )?;
            if size.iter().any(|n| *n <= 0.0) {
                return Err("Les demi-dimensions doivent être positives".into());
            }
            let p = [p[0], p[2], -p[1]];
            let size = [size[0], size[2], size[1]];
            let corners: Vec<[f32; 3]> = (0..8)
                .map(|i| {
                    std::array::from_fn(|a| {
                        ((p[a] + if i & (1 << a) == 0 { -size[a] } else { size[a] }) * scale[a])
                            as f32
                    })
                })
                .collect();
            if corners.iter().flatten().any(|n| !n.is_finite()) {
                return Err("Boîte de collision hors limites".into());
            }
            let mut vertices = Vec::with_capacity(24);
            for i in 0..8 {
                for a in 0..3 {
                    let j = i ^ (1 << a);
                    if i < j {
                        vertices.extend([corners[i], corners[j]]);
                    }
                }
            }
            Ok(crate::CollisionWire {
                section,
                name: info.name.clone(),
                vertices,
            })
        })();
        match result {
            Ok(wire) => scene.collisions.push(wire),
            Err(error) => scene.warnings.push(format!("{} : {error}", info.name)),
        }
    }
}
