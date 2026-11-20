use base64::Engine;
use cpm_core::{Document, ExportFormat, ExportOptions, ModelFiles, ObjRequest, Pack, PackMetadata};
use std::{collections::BTreeMap, fs, path::PathBuf};

fn fixture() -> (tempfile::TempDir, serde_json::Value) {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("pack_info.dynx"),
        "PackName: Demo\nDcFileVersion: 12.5.0\n",
    )
    .unwrap();
    fs::create_dir_all(temp.path().join("models")).unwrap();
    let mut buffer = Vec::new();
    for value in [0f32, 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 1.] {
        buffer.extend(value.to_le_bytes());
    }
    for value in [0u16, 1, 2] {
        buffer.extend(value.to_le_bytes());
    }
    fs::write(temp.path().join("models/geometry.bin"), buffer).unwrap();
    image::RgbaImage::from_fn(2, 2, |x, y| {
        image::Rgba([
            if x == 0 { 255 } else { 0 },
            if y == 0 { 255 } else { 0 },
            0,
            255,
        ])
    })
    .save(temp.path().join("models/paint.png"))
    .unwrap();
    let json = serde_json::json!({
        "asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],
        "nodes":[{"name":"Parent","translation":[5,0,0],"children":[1]},{"name":"chassis","mesh":0,"translation":[0,2,0],"scale":[2,1,1]}],
        "buffers":[{"uri":"geometry.bin","byteLength":66}],
        "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":36},{"buffer":0,"byteOffset":36,"byteLength":24},{"buffer":0,"byteOffset":60,"byteLength":6}],
        "accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[1,1,0]},{"bufferView":1,"componentType":5126,"count":3,"type":"VEC2"},{"bufferView":2,"componentType":5123,"count":3,"type":"SCALAR"}],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0,"TEXCOORD_0":1},"indices":2,"material":0}]}],
        "images":[{"uri":"paint.png"}],"textures":[{"source":0}],"materials":[{"name":"paint","pbrMetallicRoughness":{"baseColorFactor":[0.5,1,1,1],"baseColorTexture":{"index":0}}}]
    });
    fs::write(
        temp.path().join("models/car.gltf"),
        serde_json::to_vec(&json).unwrap(),
    )
    .unwrap();
    (temp, json)
}
fn request(pack: &Pack) -> ObjRequest {
    ObjRequest::resolve(
        Some(pack),
        std::path::Path::new("vehicle_car.dynx"),
        &Document::parse("Model: models/car.gltf\n").unwrap(),
    )
    .unwrap()
}
#[test]
fn gltf_preview_preserves_named_parts_transforms_uvs_and_base_color_texture() {
    let (temp, _) = fixture();
    let pack = Pack::open(temp.path()).unwrap();
    let mesh = request(&pack).load().unwrap();
    assert_eq!(mesh.triangles, 1);
    assert_eq!(mesh.parts[0].name, "chassis");
    assert_eq!(&mesh.vertices[0][..3], &mesh.normalize_point([5., 2., 0.]));
    assert_eq!(&mesh.vertices[1][..3], &mesh.normalize_point([7., 2., 0.]));
    assert_eq!(&mesh.vertices[0][6..], &[0., 0.]);
    assert_eq!(&mesh.vertices[2][6..], &[0., 1.]);
    assert_eq!(mesh.materials[0].diffuse, [0.5, 1., 1., 1.]);
    assert_eq!(mesh.textures[0].width, 2);
    assert!(mesh.warnings.iter().any(|w| w.contains("bêta")));
    assert!(pack
        .model_resources()
        .contains(&PathBuf::from("models/car.gltf")));
    assert!(!pack
        .obj_resources()
        .contains(&PathBuf::from("models/car.gltf")));
}
#[test]
fn gltf_import_requires_beta_and_selected_dependencies_then_exports_all_formats() {
    let (source, _) = fixture();
    let original = fs::read(source.path().join("models/car.gltf")).unwrap();
    let mut files = ModelFiles {
        obj: Some(source.path().join("models/car.gltf")),
        ..Default::default()
    };
    assert!(files
        .prepare(std::path::Path::new("models/imported"))
        .unwrap_err()
        .contains("bêta"));
    files.dynamx_beta_confirmed = true;
    assert!(files
        .prepare(std::path::Path::new("models/imported"))
        .unwrap_err()
        .contains("manquante"));
    files
        .materials
        .push(source.path().join("models/geometry.bin"));
    files.textures.push(source.path().join("models/paint.png"));
    let staged = files
        .prepare(std::path::Path::new("models/imported"))
        .unwrap();
    assert_eq!(staged.model, PathBuf::from("models/imported/model.gltf"));
    assert_eq!(staged.assets.len(), 3);
    let dest = tempfile::tempdir().unwrap();
    let blank = tempfile::tempdir().unwrap();
    fs::write(
        blank.path().join("pack_info.dynx"),
        "PackName: Demo\nDcFileVersion: 12.5.0\n",
    )
    .unwrap();
    let mut pack = Pack::open(blank.path()).unwrap();
    pack.add_bundle(vec![], staged.assets).unwrap();
    for (i, format) in [
        ExportFormat::Folder,
        ExportFormat::Zip,
        ExportFormat::Dnxpack,
    ]
    .into_iter()
    .enumerate()
    {
        let output = pack
            .export(
                &ExportOptions {
                    destination: dest.path().into(),
                    output_name: format!("out{i}"),
                    format,
                    replace_existing: false,
                    metadata: PackMetadata {
                        name: "Demo".into(),
                        version: "1.0".into(),
                        description: String::new(),
                    },
                },
                &BTreeMap::new(),
            )
            .unwrap();
        let reopened = Pack::open(&output).unwrap();
        let doc = Document::parse("Model: models/imported/model.gltf\n").unwrap();
        let mesh = ObjRequest::resolve(
            Some(&reopened),
            std::path::Path::new("item_test.dynx"),
            &doc,
        )
        .unwrap()
        .load()
        .unwrap();
        assert_eq!(mesh.triangles, 1);
        assert_eq!(mesh.textures.len(), 1);
    }
    assert_eq!(
        fs::read(source.path().join("models/car.gltf")).unwrap(),
        original
    );
}
#[test]
fn gltf_supports_embedded_buffers_and_rejects_remote_escaping_or_truncated_data() {
    let (temp, mut json) = fixture();
    let bytes = fs::read(temp.path().join("models/geometry.bin")).unwrap();
    json["buffers"][0]["uri"] = format!(
        "data:application/octet-stream;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
    .into();
    fs::write(
        temp.path().join("models/car.gltf"),
        serde_json::to_vec(&json).unwrap(),
    )
    .unwrap();
    assert_eq!(
        request(&Pack::open(temp.path()).unwrap())
            .load()
            .unwrap()
            .triangles,
        1
    );
    for uri in [
        "https://example.com/model.bin",
        "%2fetc/passwd",
        "../../outside.bin",
        "..%2f..%2foutside.bin",
    ] {
        json["buffers"][0]["uri"] = uri.into();
        fs::write(
            temp.path().join("models/car.gltf"),
            serde_json::to_vec(&json).unwrap(),
        )
        .unwrap();
        assert!(
            request(&Pack::open(temp.path()).unwrap()).load().is_err(),
            "{uri}"
        );
    }
    json["buffers"][0]["uri"] = "geometry.bin".into();
    fs::write(temp.path().join("models/geometry.bin"), [0u8; 4]).unwrap();
    fs::write(
        temp.path().join("models/car.gltf"),
        serde_json::to_vec(&json).unwrap(),
    )
    .unwrap();
    assert!(request(&Pack::open(temp.path()).unwrap()).load().is_err());
    let glb = Document::parse("Model: model.glb\n").unwrap();
    assert!(ObjRequest::resolve(
        Some(&Pack::open(temp.path()).unwrap()),
        std::path::Path::new("x.dynx"),
        &glb
    )
    .is_err());
}

#[test]
fn malformed_accessors_cycles_and_required_extensions_fail_without_panics() {
    let (temp, json) = fixture();
    let mut cases = Vec::new();
    let mut bad = json.clone();
    bad["accessors"][0]["count"] = 100.into();
    cases.push(bad);
    let mut bad = json.clone();
    bad["nodes"][1]["children"] = serde_json::json!([0]);
    cases.push(bad);
    let mut bad = json;
    bad["extensionsRequired"] = serde_json::json!(["KHR_draco_mesh_compression"]);
    cases.push(bad);
    for bad in cases {
        fs::write(
            temp.path().join("models/car.gltf"),
            serde_json::to_vec(&bad).unwrap(),
        )
        .unwrap();
        assert!(request(&Pack::open(temp.path()).unwrap()).load().is_err());
    }
}

#[test]
fn gltf_vehicles_assemble_obj_wheels_and_declared_collision_boxes() {
    let (temp, _) = fixture();
    fs::write(
        temp.path().join("models/wheel.obj"),
        "o tire\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("wheel_front.dynx"),
        "Model: models/wheel.obj\n",
    )
    .unwrap();
    let pack = Pack::open(temp.path()).unwrap();
    let doc=Document::parse("Model: models/car.gltf\nWheelFront{\nAttachedWheel: Demo.wheel_front\nPosition: 2 0 0\nIsRight: false\n}\nShape_main{\nType: BOX\nPosition: 0 0 0\nScale: 1 1 1\n}\n").unwrap();
    let mesh = ObjRequest::resolve(Some(&pack), std::path::Path::new("vehicle_car.dynx"), &doc)
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(mesh.collisions.len(), 1);
    assert!(
        mesh.parts.iter().any(|p| p.name == "WheelFront/tire"),
        "{:?}",
        mesh.warnings
    );
    assert_eq!(mesh.triangles, 2);
}
