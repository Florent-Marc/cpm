use cpm_core::{ExportFormat, ExportOptions, Pack, PackMetadata, ParameterCatalog};
use std::{collections::BTreeMap, fs, io::Write, path::Path};
fn source() -> (tempfile::TempDir, Pack) {
    let t = tempfile::tempdir().unwrap();
    for dir in ["models", "materials", "textures"] {
        fs::create_dir(t.path().join(dir)).unwrap();
    }
    fs::write(
        t.path().join("pack_info.dynx"),
        "PackName: Demo\nPackVersion: 1.0\nDcFileVersion: 12.5.0\n",
    )
    .unwrap();
    fs::write(t.path().join("vehicle_car.dynx"),"Name: Car\nDescription: Example\nModel: models/car.obj\nEmptyMass: 1000\nDragCoefficient: 0.3\nCenterOfGravityOffset: 0 0 0\nDefaultEngine: Demo.engine_main\n").unwrap();
    fs::write(
        t.path().join("engine_main.dynx"),
        "Power: 1000\nMaxRPM: 6000\nBraking: 100\n",
    )
    .unwrap();
    fs::write(t.path().join("models/car.obj"),"mtllib ../materials/body.mtl\no body\nv 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nusemtl Paint\nf 1/1 2/1 3/1\n").unwrap();
    fs::write(
        t.path().join("materials/body.mtl"),
        "newmtl Paint\nmap_Kd ../textures/red.png\n",
    )
    .unwrap();
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(t.path().join("textures/red.png"))
        .unwrap();
    fs::write(t.path().join("README.md"), "Documentation").unwrap();
    let pack = Pack::open(t.path()).unwrap();
    (t, pack)
}
#[test]
fn publication_moves_private_resources_only_in_a_copy_and_preserves_offline_export() {
    let (root, pack) = source();
    let catalogue = ParameterCatalog::builtin();
    let index = pack
        .configs()
        .iter()
        .position(|p| p == Path::new("vehicle_car.dynx"))
        .unwrap();
    let mut doc = pack.read_config(index).unwrap();
    doc.set_value(0, "Unsaved car").unwrap();
    let documents = BTreeMap::from([(Path::new("vehicle_car.dynx").into(), doc)]);
    let prepared = pack
        .prepare_openmps("demo", "2.0", &documents, &catalogue)
        .unwrap();
    assert_eq!(prepared.report.moved.len(), 3);
    assert!(prepared
        .report
        .skipped
        .iter()
        .any(|p| p == Path::new("README.md")));
    let published = Pack::open(&prepared.archive).unwrap();
    assert!(published.contains_resource(Path::new("assets/demo/models/car.obj")));
    assert!(!published.contains_resource(Path::new("models/car.obj")));
    let vehicle = published
        .read_config(
            published
                .configs()
                .iter()
                .position(|p| p == Path::new("vehicle_car.dynx"))
                .unwrap(),
        )
        .unwrap();
    assert!(vehicle.source().contains("Name: Unsaved car"));
    assert!(vehicle.source().contains("Model: demo:models/car.obj"));
    assert!(vehicle.source().contains("DefaultEngine: demo.engine_main"));
    let mesh =
        cpm_core::ObjRequest::resolve(Some(&published), Path::new("vehicle_car.dynx"), &vehicle)
            .unwrap()
            .load()
            .unwrap();
    assert!(mesh.warnings.is_empty(), "{:?}", mesh.warnings);
    assert_eq!(mesh.textures.len(), 1);
    assert!(!root.path().join("assets").exists());
    assert!(fs::read_to_string(root.path().join("vehicle_car.dynx"))
        .unwrap()
        .contains("DefaultEngine: Demo.engine_main"));
    assert!(!pack.has_structure_changes());
    let out = tempfile::tempdir().unwrap();
    let target = pack
        .export(
            &ExportOptions {
                destination: out.path().into(),
                output_name: "offline".into(),
                format: ExportFormat::Zip,
                replace_existing: false,
                metadata: PackMetadata {
                    name: "Demo".into(),
                    version: "1.0".into(),
                    description: String::new(),
                },
            },
            &documents,
        )
        .unwrap();
    let offline = Pack::open(&target).unwrap();
    assert!(offline.contains_resource(Path::new("models/car.obj")));
    assert!(!offline.contains_resource(Path::new("assets/demo/models/car.obj")));
}
#[test]
fn mps_conflicts_and_unsupported_files_do_not_block_ordinary_exports() {
    let (root, _) = source();
    fs::create_dir_all(root.path().join("assets/demo/models")).unwrap();
    fs::write(root.path().join("assets/demo/models/car.obj"), "conflict").unwrap();
    let pack = Pack::open(root.path()).unwrap();
    assert!(pack
        .prepare_openmps(
            "demo",
            "2.0",
            &BTreeMap::new(),
            &ParameterCatalog::builtin()
        )
        .is_err());
    assert!(pack
        .prepare_openmps(
            "../bad",
            "2.0",
            &BTreeMap::new(),
            &ParameterCatalog::builtin()
        )
        .is_err());
    assert!(cpm_core::validate_mps_version("../2").is_err());
    let out = tempfile::tempdir().unwrap();
    assert!(pack
        .export(
            &ExportOptions {
                destination: out.path().into(),
                output_name: "offline".into(),
                format: ExportFormat::Zip,
                replace_existing: false,
                metadata: PackMetadata {
                    name: "Demo".into(),
                    version: "1.0".into(),
                    description: String::new()
                }
            },
            &BTreeMap::new()
        )
        .is_ok());
}
#[test]
fn players_zip_must_reference_the_requested_version_and_exclude_resources() {
    let t = tempfile::tempdir().unwrap();
    let make = |path: &Path, resource: bool| {
        let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        for (name, text) in [
            (
                "pack_info.dynx",
                "PackName: demo\nPackVersion: 2.0\nDcFileVersion: 12.5.0\n",
            ),
            (
                "MpsRepositories.json",
                "{\"repositories\":[{\"key\":\"demo\",\"version\":\"2.0\"}]}",
            ),
        ] {
            zip.start_file(name, options).unwrap();
            zip.write_all(text.as_bytes()).unwrap();
        }
        if resource {
            zip.start_file("assets/demo/car.obj", options).unwrap();
            zip.write_all(b"private").unwrap();
        }
        zip.finish().unwrap();
    };
    let valid = t.path().join("valid.zip");
    make(&valid, false);
    assert!(cpm_core::validate_mps_client_archive(&valid, "demo", "2.0").is_ok());
    assert!(cpm_core::validate_mps_client_archive(&valid, "demo", "3.0").is_err());
    let leak = t.path().join("leak.zip");
    make(&leak, true);
    assert!(cpm_core::validate_mps_client_archive(&leak, "demo", "2.0").is_err());
}
