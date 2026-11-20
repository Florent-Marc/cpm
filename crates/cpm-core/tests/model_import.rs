use cpm_core::{Document, ExportFormat, ExportOptions, ModelFiles, ObjRequest, Pack, PackMetadata};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
fn metadata() -> Document {
    Document::parse("PackName: Demo\nDcFileVersion: 12.5.0\n").unwrap()
}
fn files(root: &Path) -> ModelFiles {
    fs::write(root.join("car.obj"),"mtllib ../external/old.mtl\no body\nv 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nusemtl paint\nf 1/1 2/2 3/3\n").unwrap();
    fs::write(
        root.join("chosen.mtl"),
        "newmtl Paint\nKd 1 1 1\nmap_Kd ../textures/paint.png\n",
    )
    .unwrap();
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(root.join("paint.png"))
        .unwrap();
    ModelFiles {
        obj: Some(root.join("car.obj")),
        materials: vec![root.join("chosen.mtl")],
        textures: vec![root.join("paint.png")],
        untextured: false,
        dynamx_beta_confirmed: false,
    }
}
#[test]
fn selected_assets_are_relinked_staged_and_exported_without_changing_sources() {
    let source = tempfile::tempdir().unwrap();
    let original = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    fs::write(original.path().join("pack_info.dynx"), metadata().source()).unwrap();
    let selected = files(source.path());
    let before = fs::read(source.path().join("car.obj")).unwrap();
    for format in [
        ExportFormat::Folder,
        ExportFormat::Zip,
        ExportFormat::Dnxpack,
    ] {
        let model = selected.prepare(Path::new("models/car")).unwrap();
        let doc = Document::parse(&format!("Model: {}\n", model.model.display())).unwrap();
        let mut pack = Pack::open(original.path()).unwrap();
        pack.add_bundle(
            vec![(PathBuf::from("vehicle_car.dynx"), doc.clone())],
            model.assets,
        )
        .unwrap();
        assert_eq!(pack.staged_resource_count(), 3);
        assert!(pack.has_structure_changes());
        assert_eq!(
            pack.obj_resources(),
            vec![PathBuf::from("models/car/model.obj")]
        );
        let mesh = ObjRequest::resolve(Some(&pack), Path::new("vehicle_car.dynx"), &doc)
            .unwrap()
            .load()
            .unwrap();
        assert!(mesh.warnings.is_empty(), "{:?}", mesh.warnings);
        assert_eq!(mesh.textures.len(), 1);
        let out = pack
            .export_with_catalog(
                &ExportOptions {
                    destination: destination.path().into(),
                    output_name: format!("out_{format:?}"),
                    format,
                    replace_existing: false,
                    metadata: PackMetadata::from_document(&metadata()),
                },
                &BTreeMap::new(),
                &cpm_core::ParameterCatalog { files: Vec::new() },
            )
            .unwrap();
        let exported = Pack::open(&out).unwrap();
        assert_eq!(exported.staged_resource_count(), 0);
        assert!(exported.contains_resource(Path::new("models/car/model.obj")));
        assert_eq!(exported.configs().len(), 2);
    }
    assert_eq!(fs::read(source.path().join("car.obj")).unwrap(), before);
    assert!(!original.path().join("models").exists());
    assert!(!original.path().join("vehicle_car.dynx").exists());
}
#[test]
fn missing_dependencies_duplicate_paths_and_failed_batches_are_actionable_and_atomic() {
    let root = tempfile::tempdir().unwrap();
    let mut selected = files(root.path());
    selected.textures.clear();
    assert!(selected
        .prepare(Path::new("models/car"))
        .unwrap_err()
        .contains("Texture manquante"));
    selected.untextured = true;
    let model = selected.prepare(Path::new("models/car")).unwrap();
    let obj = String::from_utf8(model.assets[&model.model].clone()).unwrap();
    assert!(!obj.contains("mtllib"));
    assert!(!obj.contains("usemtl"));
    fs::write(root.path().join("pack_info.dynx"), metadata().source()).unwrap();
    let mut pack = Pack::open(root.path()).unwrap();
    let before = pack.configs().to_vec();
    let mut assets = BTreeMap::new();
    assets.insert(PathBuf::from("models/new.obj"), b"data".to_vec());
    assert!(pack
        .add_bundle(vec![(PathBuf::from("pack_info.dynx"), metadata())], assets)
        .is_err());
    assert_eq!(pack.configs(), before);
    assert!(!pack.contains_resource(Path::new("models/new.obj")));
    assert!(!pack.has_structure_changes());
    let mut assets = BTreeMap::new();
    assets.insert(PathBuf::from("MODELS/X.obj"), vec![]);
    assets.insert(PathBuf::from("models/x.OBJ"), vec![]);
    assert!(pack.add_bundle(vec![], assets).is_err());
    let mut assets = BTreeMap::new();
    assets.insert(PathBuf::from("../escape.obj"), vec![]);
    assert!(pack.add_bundle(vec![], assets).is_err());
    assert!(selected.prepare(Path::new("../escape")).is_err());
}

#[test]
fn multiple_mtls_and_texture_names_with_spaces_remain_linked_in_preview() {
    let source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    fs::write(target.path().join("pack_info.dynx"), metadata().source()).unwrap();
    fs::write(source.path().join("car.obj"),"mtllib old.mtl\no body\nv 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nusemtl red\nf 1/1 2/1 3/1\nusemtl blue\nf 1/1 2/1 3/1\n").unwrap();
    fs::write(
        source.path().join("red.mtl"),
        "newmtl red\nmap_Kd ../textures/red paint.png\n",
    )
    .unwrap();
    fs::write(
        source.path().join("blue.mtl"),
        "newmtl blue\nmap_Kd blue.png\n",
    )
    .unwrap();
    for (name, color) in [
        ("red paint.png", [255, 0, 0, 255]),
        ("blue.png", [0, 0, 255, 255]),
    ] {
        image::RgbaImage::from_pixel(1, 1, image::Rgba(color))
            .save(source.path().join(name))
            .unwrap();
    }
    let files = ModelFiles {
        obj: Some(source.path().join("car.obj")),
        materials: vec![
            source.path().join("red.mtl"),
            source.path().join("blue.mtl"),
        ],
        textures: vec![
            source.path().join("red paint.png"),
            source.path().join("blue.png"),
        ],
        untextured: false,
        dynamx_beta_confirmed: false,
    };
    let prepared = files.prepare(Path::new("models/car")).unwrap();
    let doc = Document::parse(&format!("Model: {}\n", prepared.model.display())).unwrap();
    let mut pack = Pack::open(target.path()).unwrap();
    pack.add_bundle(
        vec![(PathBuf::from("vehicle_car.dynx"), doc.clone())],
        prepared.assets,
    )
    .unwrap();
    let mesh = ObjRequest::resolve(Some(&pack), Path::new("vehicle_car.dynx"), &doc)
        .unwrap()
        .load()
        .unwrap();
    assert!(mesh.warnings.is_empty(), "{:?}", mesh.warnings);
    assert_eq!(mesh.materials.len(), 2);
    assert_eq!(mesh.textures.len(), 2);
    assert_eq!(mesh.parts[0].batches.len(), 2);
}
