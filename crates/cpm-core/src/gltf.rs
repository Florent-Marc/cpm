//! Static glTF 2.0 geometry. Resource reads are supplied by the pack/importer, never HTTP.
use crate::{ObjBatch, ObjMaterial, ObjMesh, ObjPart, ObjTexture};
use base64::Engine;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
const MAX_BYTES: usize = 256 * 1024 * 1024;
const MAX_VERTICES: usize = 3_000_000;

fn decode_uri(uri: &str) -> Result<String, String> {
    let mut decoded = Vec::new();
    let mut bytes = uri.bytes();
    while let Some(c) = bytes.next() {
        if c == b'%' {
            let a = bytes
                .next()
                .and_then(|b| (b as char).to_digit(16))
                .ok_or("URI glTF invalide")?;
            let b = bytes
                .next()
                .and_then(|b| (b as char).to_digit(16))
                .ok_or("URI glTF invalide")?;
            decoded.push((a * 16 + b) as u8);
        } else {
            decoded.push(c);
        }
    }
    let value = String::from_utf8(decoded).map_err(|_| "URI glTF non UTF-8")?;
    if value.contains([':', '?', '#', '\\', '\0']) || value.starts_with('/') {
        return Err("Le glTF doit utiliser des ressources locales relatives (pas de réseau ni de chemin absolu)".into());
    }
    Ok(value)
}

pub(crate) fn uri_path(base: &Path, uri: &str) -> Result<PathBuf, String> {
    crate::model::related_path(base, &decode_uri(uri)?)
}
fn data_uri(uri: &str) -> Result<Vec<u8>, String> {
    let (header, data) = uri.split_once(',').ok_or("Data URI glTF invalide")?;
    if !header.ends_with(";base64") || data.len() > 180 * 1024 * 1024 {
        return Err("Data URI glTF non prise en charge ou trop volumineuse".into());
    }
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|_| "Base64 glTF invalide".into())
}
pub(crate) fn load(
    bytes: &[u8],
    path: &Path,
    mut read: impl FnMut(&Path, u64) -> Result<Vec<u8>, String>,
) -> Result<ObjMesh, String> {
    if bytes.len() > 32 * 1024 * 1024 || bytes.starts_with(b"glTF") {
        return Err(
            "Utiliser un .gltf JSON de moins de 32 Mio ; GLB non pris en charge par DynamX".into(),
        );
    }
    let gltf = gltf::Gltf::from_slice(bytes).map_err(|e| format!("glTF 2.0 invalide : {e}"))?;
    if gltf.extensions_required().next().is_some() {
        return Err("Extensions glTF obligatoires non prises en charge (exporter sans compression Draco/Meshopt ni extension requise)".into());
    }
    if gltf.nodes().len() > 10000 || gltf.materials().len() > 4096 || gltf.images().len() > 1024 {
        return Err("glTF trop complexe pour l’aperçu".into());
    }
    let mut budget = bytes.len();
    let mut resource = |uri: &str, limit: u64| -> Result<Vec<u8>, String> {
        let data = if uri.starts_with("data:") {
            data_uri(uri)?
        } else {
            read(&uri_path(path, uri)?, limit)?
        };
        budget = budget
            .checked_add(data.len())
            .ok_or("Ressources glTF trop volumineuses")?;
        if data.len() as u64 > limit || budget > MAX_BYTES {
            return Err("Ressources glTF limitées à 256 Mio".into());
        }
        Ok(data)
    };
    let mut buffers = Vec::new();
    for b in gltf.buffers() {
        if b.length() > 128 * 1024 * 1024 {
            return Err("Buffer glTF supérieur à 128 Mio".into());
        }
        let gltf::buffer::Source::Uri(uri) = b.source() else {
            return Err("Buffer GLB non pris en charge".into());
        };
        let data = resource(uri, 128 * 1024 * 1024)?;
        if data.len() < b.length() {
            return Err("Buffer glTF tronqué".into());
        }
        buffers.push(data);
    }
    for view in gltf.views() {
        if view
            .offset()
            .checked_add(view.length())
            .is_none_or(|end| end > buffers[view.buffer().index()].len())
        {
            return Err("Vue de buffer glTF hors limites".into());
        }
    }
    for a in gltf.accessors() {
        if a.count() > MAX_VERTICES {
            return Err("Trop de sommets glTF".into());
        }
        if let Some(v) = a.view() {
            let stride = v.stride().unwrap_or(a.size());
            let size = a
                .count()
                .saturating_sub(1)
                .checked_mul(stride)
                .and_then(|n| n.checked_add(a.size()))
                .and_then(|n| n.checked_add(a.offset()))
                .ok_or("Accessor glTF hors limites")?;
            if size > v.length() {
                return Err("Accessor glTF hors limites".into());
            }
        }
        if a.sparse().is_some() {
            return Err(
                "Accessors glTF sparse non pris en charge ; exporter une géométrie développée"
                    .into(),
            );
        }
    }
    let mut textures = Vec::<ObjTexture>::new();
    let mut decoded = 0usize;
    for image in gltf.images() {
        let (name, data) = match image.source() {
            gltf::image::Source::Uri { uri, .. } => (
                path.with_extension(format!("gltf-image-{}.png", image.index())),
                resource(uri, 32 * 1024 * 1024)?,
            ),
            gltf::image::Source::View { view, .. } => {
                if view.length() > 32 * 1024 * 1024 {
                    return Err("Image glTF trop volumineuse".into());
                }
                (
                    path.with_extension(format!("gltf-image-{}.png", image.index())),
                    buffers[view.buffer().index()][view.offset()..view.offset() + view.length()]
                        .to_vec(),
                )
            }
        };
        let texture =
            crate::model::decode_texture(&name, data).map_err(|e| format!("Texture glTF : {e}"))?;
        decoded += texture.rgba.len();
        if decoded > 256 * 1024 * 1024 {
            return Err("Textures glTF décodées limitées à 256 Mio".into());
        }
        textures.push(texture);
    }
    let mut warnings=vec!["glTF : DynamX bêta requis. Aperçu statique : textures de base et couleurs ; animations, skinning, morphs et rendu PBR non simulés.".into()];
    let mut materials = Vec::new();
    for m in gltf.materials() {
        let pbr = m.pbr_metallic_roughness();
        let texture = pbr.base_color_texture();
        if texture.as_ref().is_some_and(|t| t.tex_coord() != 0) {
            warnings.push("Texture glTF avec UV autres que TEXCOORD_0 : non affichée".into());
        }
        materials.push(ObjMaterial {
            name: m
                .name()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Matériau {}", m.index().unwrap_or(0))),
            diffuse: pbr.base_color_factor(),
            texture: texture
                .filter(|t| t.tex_coord() == 0)
                .map(|t| t.texture().source().index()),
        });
    }
    let mut mesh = ObjMesh {
        center: [0.0; 3],
        radius: 1.0,
        vertices: Vec::new(),
        collisions: Vec::new(),
        parts: Vec::new(),
        materials,
        textures,
        warnings,
        triangles: 0,
        objects: 0,
    };
    let roots: Vec<_> = if let Some(scene) = gltf.default_scene().or_else(|| gltf.scenes().next()) {
        scene.nodes().collect()
    } else {
        let children: BTreeSet<_> = gltf
            .nodes()
            .flat_map(|n| n.children().map(|n| n.index()).collect::<Vec<_>>())
            .collect();
        gltf.nodes()
            .filter(|n| !children.contains(&n.index()))
            .collect()
    };
    let identity = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ];
    let mut visited = BTreeSet::new();
    for node in roots {
        visit(node, identity, &buffers, &mut mesh, &mut visited, 0)?;
    }
    if mesh.vertices.is_empty() {
        return Err("Le glTF ne contient aucun triangle affichable".into());
    }
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for v in &mesh.vertices {
        for i in 0..3 {
            min[i] = min[i].min(v[i] as f64);
            max[i] = max[i].max(v[i] as f64);
        }
    }
    mesh.center = std::array::from_fn(|i| (min[i] + max[i]) * 0.5);
    mesh.radius = mesh
        .vertices
        .iter()
        .map(|v| {
            (0..3)
                .map(|i| (v[i] as f64 - mesh.center[i]).powi(2))
                .sum::<f64>()
                .sqrt()
        })
        .fold(0.0, f64::max);
    if mesh.radius <= 0.0 {
        return Err("Le glTF a une taille nulle".into());
    }
    for v in &mut mesh.vertices {
        for (i, value) in v.iter_mut().take(3).enumerate() {
            *value = ((*value as f64 - mesh.center[i]) / mesh.radius) as f32;
        }
    }
    mesh.triangles = mesh.vertices.len() / 3;
    mesh.objects = mesh.parts.len();
    Ok(mesh)
}
fn visit(
    node: gltf::Node<'_>,
    parent: [[f64; 4]; 4],
    buffers: &[Vec<u8>],
    output: &mut ObjMesh,
    visited: &mut BTreeSet<usize>,
    depth: usize,
) -> Result<(), String> {
    if depth > 128 || !visited.insert(node.index()) {
        return Err("Hiérarchie glTF cyclique, partagée ou trop profonde".into());
    }
    let local = node.transform().matrix();
    let world = std::array::from_fn::<_, 4, _>(|col| {
        std::array::from_fn(|row| {
            (0..4)
                .map(|k| parent[k][row] * local[col][k] as f64)
                .sum::<f64>()
        })
    });
    if world.iter().flatten().any(|v| !v.is_finite()) {
        return Err("Transformation glTF invalide".into());
    }
    if let Some(mesh) = node.mesh() {
        let name = node
            .name()
            .or(mesh.name())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("Node_{}", node.index()));
        let mut part = ObjPart {
            section: None,
            name,
            batches: Vec::new(),
        };
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                output
                    .warnings
                    .push(format!("{} : primitive non triangulée ignorée", part.name));
                continue;
            }
            let reader = primitive.reader(|b| buffers.get(b.index()).map(Vec::as_slice));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or("Positions glTF absentes")?
                .collect();
            let indices: Vec<u32> = reader
                .read_indices()
                .map(|i| i.into_u32().collect())
                .unwrap_or_else(|| (0..positions.len() as u32).collect());
            if !indices.len().is_multiple_of(3)
                || output.vertices.len().saturating_add(indices.len()) > MAX_VERTICES
            {
                return Err("Triangles glTF invalides ou limite d’un million dépassée".into());
            }
            let uvs: Option<Vec<_>> = reader.read_tex_coords(0).map(|v| v.into_f32().collect());
            let first = output.vertices.len();
            for face in indices.as_chunks::<3>().0 {
                let mut points = [[0f32; 3]; 3];
                let mut tex = [[0f32; 2]; 3];
                for (i, index) in face.iter().enumerate() {
                    let p = positions
                        .get(*index as usize)
                        .ok_or("Indice glTF hors limites")?;
                    points[i] = std::array::from_fn(|r| {
                        (world[3][r] + (0..3).map(|k| world[k][r] * p[k] as f64).sum::<f64>())
                            as f32
                    });
                    if let Some(uv) = &uvs {
                        tex[i] = *uv.get(*index as usize).ok_or("UV glTF hors limites")?;
                    }
                }
                if points
                    .iter()
                    .flatten()
                    .chain(tex.iter().flatten())
                    .any(|v| !v.is_finite())
                {
                    return Err("Coordonnées glTF non finies".into());
                }
                let a = std::array::from_fn::<_, 3, _>(|i| points[1][i] - points[0][i]);
                let b = std::array::from_fn::<_, 3, _>(|i| points[2][i] - points[0][i]);
                let n = [
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                ];
                let len = n.iter().map(|x| x * x).sum::<f32>().sqrt();
                if !len.is_finite() {
                    return Err("Normale glTF hors limites".into());
                }
                if len <= 0.0 {
                    continue;
                }
                for (p, uv) in points.into_iter().zip(tex) {
                    output.vertices.push([
                        p[0],
                        p[1],
                        p[2],
                        n[0] / len,
                        n[1] / len,
                        n[2] / len,
                        uv[0],
                        uv[1],
                    ]);
                }
            }
            if output.vertices.len() > first {
                part.batches.push(ObjBatch {
                    first,
                    count: output.vertices.len() - first,
                    material: primitive.material().index(),
                    has_uv: uvs.is_some(),
                });
            }
        }
        if !part.batches.is_empty() {
            output.parts.push(part);
        }
    }
    for child in node.children() {
        visit(child, world, buffers, output, visited, depth + 1)?;
    }
    Ok(())
}

pub(crate) fn prepare(
    files: &crate::ModelFiles,
    folder: &Path,
) -> Result<crate::PreparedModel, String> {
    if !files.dynamx_beta_confirmed {
        return Err("Confirmer l’utilisation de DynamX bêta avant d’ajouter un glTF".into());
    }
    if files.untextured {
        return Err(
            "L’import glTF conserve ses matériaux ; sélectionner les images référencées".into(),
        );
    }
    let path = files.obj.as_ref().ok_or("Choisir le glTF")?;
    let bytes = crate::model_import::read(path, 32 * 1024 * 1024)?;
    let mut json: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("glTF invalide : {e}"))?;
    let sources = crate::model_import::selected(
        &files
            .materials
            .iter()
            .chain(&files.textures)
            .cloned()
            .collect::<Vec<_>>(),
    )?
    .into_iter()
    .map(|(k, v)| (k, v.clone()))
    .collect::<BTreeMap<_, _>>();
    let mut assets = BTreeMap::new();
    let mut total = bytes.len();
    for kind in ["buffers", "images"] {
        if let Some(entries) = json[kind].as_array_mut() {
            for (i, entry) in entries.iter_mut().enumerate() {
                let Some(uri) = entry["uri"].as_str() else {
                    continue;
                };
                if uri.starts_with("data:") {
                    continue;
                }
                // Only a selected basename is read; parent segments never trigger filesystem reads.
                let relative = PathBuf::from(decode_uri(uri)?);
                let name = relative
                    .file_name()
                    .ok_or("URI glTF invalide")?
                    .to_string_lossy()
                    .to_lowercase();
                let selected=sources.get(&name).ok_or_else(||format!("Ressource glTF manquante : {uri}. Sélectionner ce fichier .bin ou cette image."))?;
                let ext = relative
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if (kind == "buffers" && ext != "bin")
                    || (kind == "images" && !matches!(ext.as_str(), "png" | "jpg" | "jpeg"))
                {
                    return Err(format!("Ressource glTF non prise en charge : {uri}"));
                }
                let data = crate::model_import::read(
                    selected,
                    if kind == "buffers" {
                        128 * 1024 * 1024
                    } else {
                        32 * 1024 * 1024
                    },
                )?;
                total += data.len();
                if total > MAX_BYTES {
                    return Err("Import glTF limité à 256 Mio".into());
                }
                let target = format!("{kind}_{i}.{ext}");
                entry["uri"] = target.clone().into();
                assets.insert(folder.join(target), data);
            }
        }
    }
    let model = folder.join("model.gltf");
    let output = serde_json::to_vec_pretty(&json).map_err(|e| e.to_string())?;
    let preview = load(&output, &model, |p, limit| {
        let b = assets
            .get(p)
            .ok_or_else(|| format!("Dépendance absente : {}", p.display()))?;
        if b.len() as u64 > limit {
            return Err("Dépendance trop volumineuse".into());
        }
        Ok(b.clone())
    })?;
    assets.insert(model.clone(), output);
    Ok(crate::PreparedModel {
        model,
        assets,
        notes: preview.warnings,
    })
}
