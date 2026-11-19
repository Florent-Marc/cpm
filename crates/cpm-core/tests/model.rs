use cpm_core::{Document, ObjMesh, ObjRequest, Pack};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
const OBJ: &str = "o Quad\nv 0 0 0\nv 2 0 0\nv 2 2 0\nv 0 2 0\nf -4 -3 -2 -1\n";
fn document(model: &str) -> Document {
    Document::parse(&format!("Model: {model}\n")).unwrap()
}
#[test]
fn parses_quads_negative_indices_multiple_objects_and_normalizes_geometry() {
    let mesh = ObjMesh::parse(OBJ.as_bytes()).unwrap();
    let with_material = format!("mtllib missing.mtl\nusemtl body\n{OBJ}");
    assert_eq!(
        ObjMesh::parse(with_material.as_bytes()).unwrap().triangles,
        2
    );
    assert_eq!(mesh.triangles, 2);
    assert_eq!(mesh.vertices.len(), 6);
    assert!(mesh
        .vertices
        .iter()
        .all(|p| p.iter().all(|v| v.is_finite())
            && p[..3].iter().map(|v| v * v).sum::<f32>() <= 1.001));
    assert!(mesh
        .vertices
        .iter()
        .all(|p| (p[5].abs() - 1.0).abs() < 0.001));
    let multi = format!("{OBJ}o Triangle\nf 1 2 3\n");
    assert_eq!(ObjMesh::parse(multi.as_bytes()).unwrap().objects, 2);
    for bad in [
        "",
        "v 0 0 0\nf 1 1 1\n",
        "v NaN 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
        "v 0 0 0\nf 1 2 3\n",
    ] {
        assert!(ObjMesh::parse(bad.as_bytes()).is_err(), "{bad}");
    }
}
#[test]
fn loads_obj_from_directory_and_both_zip_extensions_without_extraction() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pack");
    fs::create_dir_all(root.join("obj")).unwrap();
    fs::write(root.join("pack_info.dynx"), "PackName: Demo\n").unwrap();
    fs::write(root.join("obj/car.obj"), OBJ).unwrap();
    fs::write(root.join("vehicle_demo.dynx"), "Model: obj/car.obj\n").unwrap();
    for ext in ["zip", "dnxpack"] {
        let mut zip =
            zip::ZipWriter::new(fs::File::create(temp.path().join(format!("pack.{ext}"))).unwrap());
        for (path, bytes) in [("pack_info.dynx", "PackName: Demo\n"), ("obj/car.obj", OBJ)] {
            zip.start_file(path, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(bytes.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    for source in [
        root.clone(),
        temp.path().join("pack.zip"),
        temp.path().join("pack.dnxpack"),
    ] {
        let pack = Pack::open(&source).unwrap();
        let request = ObjRequest::resolve(
            Some(&pack),
            Path::new("vehicle_demo.dynx"),
            &document("obj/car.obj"),
        )
        .unwrap();
        assert_eq!(request.load().unwrap().triangles, 2);
        assert!(pack.read_resource(Path::new("obj/car.obj"), 2).is_err());
        assert!(pack
            .read_resource(Path::new("../outside.obj"), 1000)
            .is_err());
        assert!(ObjRequest::resolve(
            Some(&pack),
            Path::new("vehicle_demo.dynx"),
            &document("missing.obj")
        )
        .is_err());
    }
    assert_eq!(
        ObjRequest::resolve(
            None,
            &root.join("vehicle_demo.dynx"),
            &document("obj/car.obj")
        )
        .unwrap()
        .load()
        .unwrap()
        .triangles,
        2
    );
}
#[test]
fn resolves_namespaces_relative_configs_and_rejects_external_or_non_obj_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("assets/custom/models")).unwrap();
    fs::create_dir_all(root.join("vehicle")).unwrap();
    fs::write(root.join("pack_info.dynx"), "PackName: Demo\n").unwrap();
    fs::write(root.join("assets/custom/models/car.obj"), OBJ).unwrap();
    fs::write(root.join("vehicle/local.obj"), OBJ).unwrap();
    let pack = Pack::open(root).unwrap();
    assert_eq!(
        ObjRequest::resolve(
            Some(&pack),
            Path::new("vehicle/test.dynx"),
            &document("custom:models/car.obj")
        )
        .unwrap()
        .load()
        .unwrap()
        .triangles,
        2
    );
    assert!(ObjRequest::resolve(
        Some(&pack),
        Path::new("vehicle/test.dynx"),
        &document("local.obj")
    )
    .is_ok());
    for path in [
        "../escape.obj",
        "/absolute.obj",
        "C:\\escape.obj",
        "custom:../../escape.obj",
        "foo.gltf",
    ] {
        assert!(
            ObjRequest::resolve(Some(&pack), Path::new("vehicle/test.dynx"), &document(path))
                .is_err(),
            "{path}"
        );
    }
}
#[test]
fn rar_resources_are_read_without_materializing_the_pack() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pack.rar");
    let pack = Pack::open(&path).unwrap();
    let bytes = pack
        .read_resource(Path::new("pack_info.dynx"), 1024 * 1024)
        .unwrap();
    assert!(String::from_utf8(bytes).unwrap().contains("PackName"));
    assert!(pack.read_resource(Path::new("pack_info.dynx"), 1).is_err());
}
#[cfg(unix)]
#[test]
fn resource_symlinks_cannot_escape_pack_after_indexing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pack");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("pack_info.dynx"), "PackName: Demo\n").unwrap();
    fs::write(root.join("car.obj"), OBJ).unwrap();
    let pack = Pack::open(&root).unwrap();
    fs::remove_file(root.join("car.obj")).unwrap();
    fs::write(temp.path().join("outside.obj"), OBJ).unwrap();
    std::os::unix::fs::symlink(temp.path().join("outside.obj"), root.join("car.obj")).unwrap();
    assert!(pack.read_resource(Path::new("car.obj"), 1024).is_err());
}

fn tiny_png() -> Vec<u8> {
    let image = image::RgbaImage::from_raw(1, 2, vec![255, 0, 0, 255, 0, 0, 255, 128]).unwrap();
    let mut bytes = std::io::Cursor::new(Vec::new());
    image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    bytes.into_inner()
}
const TEXTURED_OBJ: &str = "mtllib materials/car.mtl\no body\nv 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nusemtl Red\nf 1/1 2/2 3/3\nusemtl Blue\nf 1/1 2/2 3/3\ng wheel_front\nusemtl Red\nf 1/1 2/2 3/3\n";
const TEXTURED_MTL: &str = "newmtl Red\nKd 1 0.5 0.25\nmap_Kd ../../textures/car.png\nnewmtl Blue\nKd 0 0 1\nd 0.5\nmap_Kd ../../textures/car.png\n";
#[test]
fn textures_and_named_parts_preserve_uv_material_batches_in_folders_zip_and_dnxpack() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pack");
    fs::create_dir_all(root.join("obj/materials")).unwrap();
    fs::create_dir(root.join("textures")).unwrap();
    let png = tiny_png();
    let entries: Vec<(&str, &[u8])> = vec![
        ("pack_info.dynx", b"PackName: Test\n"),
        ("obj/car.obj", TEXTURED_OBJ.as_bytes()),
        ("obj/materials/car.mtl", TEXTURED_MTL.as_bytes()),
        ("textures/car.png", &png),
    ];
    for (path, bytes) in &entries {
        fs::write(root.join(path), bytes).unwrap();
    }
    for ext in ["zip", "dnxpack"] {
        let mut zip =
            zip::ZipWriter::new(fs::File::create(temp.path().join(format!("pack.{ext}"))).unwrap());
        for (path, bytes) in &entries {
            zip.start_file(*path, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
    }
    for source in [
        root.clone(),
        temp.path().join("pack.zip"),
        temp.path().join("pack.dnxpack"),
    ] {
        let pack = Pack::open(&source).unwrap();
        let mesh = ObjRequest::resolve(
            Some(&pack),
            Path::new("vehicle_car.dynx"),
            &document("obj/car.obj"),
        )
        .unwrap()
        .load()
        .unwrap();
        assert!(mesh.warnings.is_empty(), "{:?}", mesh.warnings);
        assert_eq!(mesh.objects, 2);
        assert_eq!(
            mesh.parts
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            vec!["body", "wheel_front"]
        );
        assert_eq!(mesh.parts[0].batches.len(), 2);
        assert_eq!(mesh.parts[1].batches.len(), 1);
        assert_eq!(mesh.parts[0].batches[0].material, Some(0));
        assert_eq!(mesh.parts[0].batches[1].material, Some(1));
        assert_eq!(mesh.parts[1].batches[0].first, 6);
        assert_eq!(mesh.parts[1].batches[0].count, 3);
        assert_eq!(mesh.textures.len(), 1);
        assert_eq!(mesh.textures[0].rgba, png_pixels());
        assert!(mesh.textures[0].has_alpha);
        assert_eq!(mesh.materials[0].diffuse, [1.0, 0.5, 0.25, 1.0]);
        assert_eq!(mesh.materials[1].diffuse, [0.0, 0.0, 1.0, 0.5]);
        assert_eq!(mesh.materials[0].texture, mesh.materials[1].texture);
        assert_eq!(&mesh.vertices[0][6..], &[0.0, 1.0]);
        assert_eq!(&mesh.vertices[2][6..], &[0.0, 0.0]);
    }
}
fn png_pixels() -> Vec<u8> {
    vec![255, 0, 0, 255, 0, 0, 255, 128]
}
#[test]
fn missing_mtl_texture_and_external_references_fall_back_with_warnings() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("obj/materials")).unwrap();
    fs::write(root.join("pack_info.dynx"), "PackName: Test\n").unwrap();
    fs::write(root.join("obj/car.obj"), TEXTURED_OBJ).unwrap();
    let load = || {
        let pack = Pack::open(root).unwrap();
        ObjRequest::resolve(
            Some(&pack),
            Path::new("vehicle_car.dynx"),
            &document("obj/car.obj"),
        )
        .unwrap()
        .load()
        .unwrap()
    };
    let missing = load();
    assert_eq!(missing.triangles, 3);
    assert!(!missing.warnings.is_empty());
    fs::write(root.join("obj/materials/car.mtl"), TEXTURED_MTL).unwrap();
    let missing = load();
    assert_eq!(missing.materials.len(), 2);
    assert!(missing.textures.is_empty());
    assert!(!missing.warnings.is_empty());
    fs::write(
        root.join("obj/materials/car.mtl"),
        "newmtl Red\nmap_Kd ../../../outside.png\n",
    )
    .unwrap();
    let blocked = load();
    assert!(blocked.textures.is_empty());
    assert!(blocked
        .warnings
        .iter()
        .any(|w| w.contains("sort du dossier")));
}
#[test]
fn texture_dimensions_are_limited_and_missing_uv_is_reported() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("pack_info.dynx"), "PackName: Test\n").unwrap();
    fs::write(
        root.join("car.obj"),
        format!("mtllib car.mtl\nusemtl Material\n{OBJ}"),
    )
    .unwrap();
    fs::write(root.join("car.mtl"), "newmtl Material\nmap_Kd car.png\n").unwrap();
    fs::write(root.join("car.png"), tiny_png()).unwrap();
    let load = || {
        let pack = Pack::open(root).unwrap();
        ObjRequest::resolve(
            Some(&pack),
            Path::new("vehicle_car.dynx"),
            &document("car.obj"),
        )
        .unwrap()
        .load()
        .unwrap()
    };
    let mesh = load();
    assert_eq!(mesh.textures.len(), 1);
    assert!(mesh.warnings.iter().any(|w| w.contains("UV absentes")));
    let image = image::RgbaImage::new(4097, 1);
    image.save(root.join("car.png")).unwrap();
    let mesh = load();
    assert!(mesh.textures.is_empty());
    assert!(!mesh.warnings.is_empty());
}

#[test]
fn wheel_models_use_config_positions_scales_right_orientation_and_unsaved_documents() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("pack_info.dynx"), "PackName: Demo\n").unwrap();
    let triangle = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
    fs::write(root.join("car.obj"), format!("o body\n{triangle}")).unwrap();
    fs::write(root.join("wheel.obj"), format!("o tire\n{triangle}")).unwrap();
    let wheel = "Model: wheel.obj\nScaleModifier: 0.5 0.5 0.5\n";
    fs::write(root.join("wheel_demo.dynx"), wheel).unwrap();
    let vehicle=Document::parse("Model: car.obj\nScaleModifier: 2 2 2\nWheel_left{\nAttachedWheel: Demo.wheel_demo\nPosition: 3 4 5\nIsRight: false\n}\nWheel_right{\nAttachedWheel: Demo.wheel_demo\nPosition: -3 4 5\nIsRight: true\n}\n").unwrap();
    let pack = Pack::open(root).unwrap();
    let scene = ObjRequest::resolve(Some(&pack), Path::new("vehicle_demo.dynx"), &vehicle)
        .unwrap()
        .load()
        .unwrap();
    assert!(scene.warnings.is_empty(), "{:?}", scene.warnings);
    assert_eq!(scene.triangles, 3);
    let left = scene
        .parts
        .iter()
        .find(|p| p.name == "Wheel_left/tire")
        .unwrap()
        .batches[0]
        .first;
    let right = scene
        .parts
        .iter()
        .find(|p| p.name == "Wheel_right/tire")
        .unwrap()
        .batches[0]
        .first;
    let unit = scene.vertices[1][0] - scene.vertices[0][0];
    for (axis, expected) in [3.0, 5.0, -4.0].into_iter().enumerate() {
        assert!(
            ((scene.vertices[left][axis] - scene.vertices[0][axis]) / unit - expected).abs()
                < 0.001
        );
    }
    assert!(((scene.vertices[left + 1][0] - scene.vertices[left][0]) / unit - 0.5).abs() < 0.001);
    assert!(((scene.vertices[right + 1][0] - scene.vertices[right][0]) / unit + 0.5).abs() < 0.001);
    let mut documents = std::collections::BTreeMap::new();
    documents.insert(
        PathBuf::from("wheel_demo.dynx"),
        Document::parse("Model: wheel.obj\nScaleModifier: 2 2 2\n").unwrap(),
    );
    let edited = ObjRequest::resolve(Some(&pack), Path::new("vehicle_demo.dynx"), &vehicle)
        .unwrap()
        .with_documents(documents)
        .load()
        .unwrap();
    let unit = edited.vertices[1][0] - edited.vertices[0][0];
    assert!(((edited.vertices[left + 1][0] - edited.vertices[left][0]) / unit - 2.0).abs() < 0.001);
    let mut deleted = pack.clone();
    deleted
        .remove_configuration(Path::new("wheel_demo.dynx"))
        .unwrap();
    let missing = ObjRequest::resolve(Some(&deleted), Path::new("vehicle_demo.dynx"), &vehicle)
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(missing.triangles, 1);
    assert!(missing
        .warnings
        .iter()
        .any(|w| w.contains("référence de roue absente")));
    assert_eq!(
        fs::read_to_string(root.join("wheel_demo.dynx")).unwrap(),
        wheel
    );
}
