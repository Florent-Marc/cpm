use cpm_core::{Document, ExportFormat, ExportOptions, Pack, PackMetadata};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    fs::create_dir_all(source.join("assets/empty")).unwrap();
    fs::write(
        source.join("pack_info.dynx"),
        "// metadata\r\nPackName: Original\r\nDcFileVersion: 12.5.0\r\nCustom: keep\r\n",
    )
    .unwrap();
    fs::write(source.join("vehicle_car.dynx"), "Name: Car\nDefaultEngine: Original.engine_car\nWheel_front{\n AttachedWheel: Original.wheel_car\n}\n").unwrap();
    fs::write(source.join("engine_car.dynx"), "Power: 1000\n").unwrap();
    fs::write(source.join("assets/model.obj"), [0, 1, 255, 42]).unwrap();
    temp
}
fn options(temp: &Path, format: ExportFormat) -> ExportOptions {
    ExportOptions {
        destination: temp.to_owned(),
        output_name: "exported".into(),
        format,
        replace_existing: false,
        metadata: PackMetadata {
            name: "Renamed".into(),
            version: "2.0".into(),
            description: "Pack de test".into(),
        },
    }
}
fn read_resource(path: &Path, resource: &str) -> Vec<u8> {
    if path.is_dir() {
        fs::read(path.join(resource)).unwrap()
    } else {
        let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
        let mut bytes = Vec::new();
        archive
            .by_name(resource)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    }
}

#[test]
fn exports_all_formats_with_assets_metadata_and_every_in_memory_edit() {
    let temp = fixture();
    let pack = Pack::open(&temp.path().join("source")).unwrap();
    let original = fs::read(pack.source().join("pack_info.dynx")).unwrap();
    let mut documents = BTreeMap::new();
    documents.insert(PathBuf::from("vehicle_car.dynx"), Document::parse("Name: Edited\nDefaultEngine: Original.engine_car\nWheel_front{\n AttachedWheel: Original.wheel_car\n}\n").unwrap());
    documents.insert(
        PathBuf::from("engine_car.dynx"),
        Document::parse("Power: 3000\n").unwrap(),
    );
    for format in [
        ExportFormat::Folder,
        ExportFormat::Zip,
        ExportFormat::Dnxpack,
    ] {
        let mut config = options(temp.path(), format);
        config.output_name = format!("output-{}", format.label().trim_start_matches('.'));
        let output = pack.export(&config, &documents).unwrap();
        let reopened = Pack::open(&output).unwrap();
        assert_eq!(reopened.configs().len(), 3);
        let info = reopened.read_config(0).unwrap();
        assert_eq!(PackMetadata::from_document(&info).name, "Renamed");
        assert_eq!(PackMetadata::from_document(&info).version, "2.0");
        assert_eq!(
            PackMetadata::from_document(&info).description,
            "Pack de test"
        );
        assert!(info.source().contains("// metadata\r\n"));
        assert!(info.source().contains("Custom: keep\r\n"));
        let vehicle = String::from_utf8(read_resource(&output, "vehicle_car.dynx")).unwrap();
        assert!(vehicle.contains("Name: Edited"));
        assert!(vehicle.contains("DefaultEngine: Renamed.engine_car"));
        assert!(vehicle.contains("AttachedWheel: Renamed.wheel_car"));
        assert_eq!(read_resource(&output, "engine_car.dynx"), b"Power: 3000\n");
        assert_eq!(
            read_resource(&output, "assets/model.obj"),
            vec![0, 1, 255, 42]
        );
        if output.is_dir() {
            assert!(output.join("assets/empty").is_dir());
        }
    }
    assert_eq!(
        fs::read(pack.source().join("pack_info.dynx")).unwrap(),
        original
    );
}

#[test]
fn refuses_overwrite_unless_checked_and_can_replace_the_source_folder() {
    let temp = fixture();
    let source = temp.path().join("source");
    let pack = Pack::open(&source).unwrap();
    let mut config = options(temp.path(), ExportFormat::Folder);
    config.output_name = "source".into();
    assert!(pack.export(&config, &BTreeMap::new()).is_err());
    assert_eq!(
        PackMetadata::from_document(&pack.read_config(0).unwrap()).name,
        "Original"
    );
    config.replace_existing = true;
    pack.export(&config, &BTreeMap::new()).unwrap();
    let reopened = Pack::open(&source).unwrap();
    assert_eq!(
        PackMetadata::from_document(&reopened.read_config(0).unwrap()).name,
        "Renamed"
    );
    assert_eq!(
        fs::read(source.join("assets/model.obj")).unwrap(),
        [0, 1, 255, 42]
    );
}

#[test]
fn can_replace_an_open_zip_after_reading_its_resources() {
    let temp = fixture();
    let folder = Pack::open(&temp.path().join("source")).unwrap();
    let mut config = options(temp.path(), ExportFormat::Zip);
    let archive = folder.export(&config, &BTreeMap::new()).unwrap();
    let pack = Pack::open(&archive).unwrap();
    config.replace_existing = true;
    config.metadata.name = "Third".into();
    pack.export(&config, &BTreeMap::new()).unwrap();
    assert_eq!(
        PackMetadata::from_document(&Pack::open(&archive).unwrap().read_config(0).unwrap()).name,
        "Third"
    );
    assert_eq!(read_resource(&archive, "assets/model.obj"), [0, 1, 255, 42]);
}

#[test]
fn failed_preparation_leaves_existing_output_untouched_and_cleans_staging() {
    let temp = fixture();
    let pack = Pack::open(&temp.path().join("source")).unwrap();
    let mut config = options(temp.path(), ExportFormat::Zip);
    let output = pack.export(&config, &BTreeMap::new()).unwrap();
    let original = fs::read(&output).unwrap();
    config.replace_existing = true;
    config.metadata.description = "bad\ninjected: x".into();
    assert!(pack.export(&config, &BTreeMap::new()).is_err());
    assert_eq!(fs::read(&output).unwrap(), original);
    assert!(!fs::read_dir(temp.path()).unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".cpm-export-")));
}

#[test]
fn rejects_paths_inside_source_unsafe_names_and_foreign_overrides() {
    let temp = fixture();
    let pack = Pack::open(&temp.path().join("source")).unwrap();
    let mut config = options(pack.source(), ExportFormat::Folder);
    assert!(pack.export(&config, &BTreeMap::new()).is_err());
    config.destination = temp.path().to_owned();
    for name in ["../outside", "a/b", "C:\\outside", "..", "CON", "trailing."] {
        config.output_name = name.into();
        assert!(config.target().is_err(), "{name}");
    }
    config.output_name = "okay".into();
    let documents = BTreeMap::from([(
        PathBuf::from("../escape.dynx"),
        Document::parse("Name: x").unwrap(),
    )]);
    assert!(pack.export(&config, &documents).is_err());
}

#[test]
fn imports_real_rar_and_exports_it_without_external_programs() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("pack.rar");
    fs::write(&source, include_bytes!("fixtures/pack.rar")).unwrap();
    let pack = Pack::open(&source).unwrap();
    assert_eq!(pack.configs().len(), 2);
    assert_eq!(pack.read_config(1).unwrap().properties()[0].value, "RarCar");
    for format in [
        ExportFormat::Folder,
        ExportFormat::Zip,
        ExportFormat::Dnxpack,
    ] {
        let mut config = options(temp.path(), format);
        config.metadata.name = "RarDemo".into();
        config.output_name = format!("rar-output-{}", format.label().trim_start_matches('.'));
        let output = pack.export(&config, &BTreeMap::new()).unwrap();
        assert_eq!(read_resource(&output, "assets/model.obj"), b"v 1 2 3\n");
        assert_eq!(
            Pack::open(&output)
                .unwrap()
                .read_config(1)
                .unwrap()
                .properties()[0]
                .value,
            "RarCar"
        );
    }
    assert_eq!(
        fs::read(source).unwrap(),
        include_bytes!("fixtures/pack.rar")
    );
}

#[test]
fn duplicate_metadata_does_not_silently_choose_a_value() {
    let temp = fixture();
    let pack = Pack::open(&temp.path().join("source")).unwrap();
    let documents = BTreeMap::from([(
        PathBuf::from("pack_info.dynx"),
        Document::parse("PackName: A\nPackName: B\nDcFileVersion: 12.5.0\n").unwrap(),
    )]);
    let config = options(temp.path(), ExportFormat::Dnxpack);
    assert!(pack
        .export(&config, &documents)
        .unwrap_err()
        .contains("dupliquée"));
    assert!(!config.target().unwrap().exists());
}
