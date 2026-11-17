use cpm_core::{
    configuration_issues, parameter_catalog, Document, EditError, ExportFormat, ExportOptions,
    Pack, PackMetadata,
};
use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn add_root_keeps_original_source_and_does_not_put_property_in_a_section() {
    let source = "\u{feff}// commentaire\r\nPackName: Démo\r\nRequiredAddon{\r\n Id: test\r\n}";
    let mut document = Document::parse(source).unwrap();
    document
        .add_property(None, "DcFileVersion", "12.5.0")
        .unwrap();
    assert_eq!(
        document.source(),
        format!("{source}\r\nDcFileVersion: 12.5.0\r\n")
    );
    let property = document.properties().last().unwrap();
    assert_eq!(property.section, None);
    assert!(configuration_issues(Path::new("pack_info.dynx"), &document).is_empty());
}

#[test]
fn add_in_nested_section_preserves_comments_newlines_and_following_offsets() {
    let source = "Name: Démo\r\nSeat_front{\r\n    // } ignored\r\n    Custom{\r\n        Value: old\r\n    }\r\n}\r\nTail: end";
    let mut document = Document::parse(source).unwrap();
    document.add_property(Some(0), "Position", "0 1 2").unwrap();
    document.add_property(Some(1), "Position", "3 4 5").unwrap();
    assert_eq!(
        document
            .properties()
            .iter()
            .filter(|p| p.key == "Position")
            .map(|p| p.section)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(0)]
    );
    assert!(document.source().contains("// } ignored\r\n"));
    let last = document
        .properties()
        .iter()
        .position(|p| p.key == "Tail")
        .unwrap();
    document.set_value(last, "updated").unwrap();
    assert!(document.source().ends_with("Tail: updated"));
    assert_eq!(document.sections()[1].parent, Some(0));
}

#[test]
fn duplicate_or_injected_parameter_never_changes_source() {
    let mut document = Document::parse("Name: test\nSeat{\n}\n").unwrap();
    let source = document.source().to_owned();
    assert_eq!(
        document.add_property(None, "Name", "x"),
        Err(EditError::DuplicateProperty)
    );
    for key in ["", "//x", "key:value", "New\nInjected", "Bad Name"] {
        assert_eq!(
            document.add_property(None, key, "x"),
            Err(EditError::InvalidKey)
        );
    }
    assert_eq!(
        document.add_property(Some(9), "New", "x"),
        Err(EditError::UnknownSection(9))
    );
    assert_eq!(
        document.add_property(None, "New", "x\ny"),
        Err(EditError::MultilineValue)
    );
    assert_eq!(document.source(), source);
    document.add_property(Some(0), "Name", "scoped").unwrap();
}

#[test]
fn required_catalog_and_validation_agree_on_the_pack_info_fix() {
    let path = Path::new("pack_info.dynx");
    let mut document = Document::parse("PackName: Test\n").unwrap();
    let issues = configuration_issues(path, &document);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("DcFileVersion"));
    let spec = parameter_catalog(path, &document, None)
        .into_iter()
        .find(|p| p.key == "DcFileVersion")
        .unwrap();
    assert!(spec.required);
    document
        .add_property(None, spec.key, spec.default_value)
        .unwrap();
    assert!(configuration_issues(path, &document).is_empty());
    document.set_value(1, "").unwrap();
    assert!(!configuration_issues(path, &document).is_empty());
}

#[test]
fn scoped_required_fields_and_aliases_are_respected() {
    let mut document =
        Document::parse("Power: 1000\nMaxRPM: 5000\nBraking: 100\nPoint_1{\n}\n").unwrap();
    assert_eq!(
        configuration_issues(Path::new("engine_car.dynx"), &document).len(),
        1
    );
    document.add_property(Some(0), "RPMPower", "0 0").unwrap();
    assert!(configuration_issues(Path::new("engine_car.dynx"), &document).is_empty());
    let document =
        Document::parse("Name: Item\nDescription:\nModel: obj/item.obj\nCreativeTab: Demo\n")
            .unwrap();
    assert!(configuration_issues(Path::new("item_test.dynx"), &document).is_empty());
    assert!(configuration_issues(
        Path::new("addon_custom.dynx"),
        &Document::parse("Custom: test").unwrap()
    )
    .is_empty());
}

#[test]
fn missing_required_param_blocks_export_until_it_is_added() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("pack_info.dynx"), "PackName: Test\n").unwrap();
    let pack = Pack::open(&source).unwrap();
    let options = ExportOptions {
        destination: temp.path().to_owned(),
        output_name: "result".into(),
        format: ExportFormat::Dnxpack,
        replace_existing: false,
        metadata: PackMetadata {
            name: "Test".into(),
            version: "1.0".into(),
            description: String::new(),
        },
    };
    assert!(pack
        .export(&options, &BTreeMap::new())
        .unwrap_err()
        .contains("DcFileVersion"));
    assert!(!options.target().unwrap().exists());
    let mut document = pack.read_config(0).unwrap();
    document
        .add_property(None, "DcFileVersion", "12.5.0")
        .unwrap();
    let documents = BTreeMap::from([(std::path::PathBuf::from("pack_info.dynx"), document)]);
    let output = pack.export(&options, &documents).unwrap();
    assert!(configuration_issues(
        Path::new("pack_info.dynx"),
        &Pack::open(&output).unwrap().read_config(0).unwrap()
    )
    .is_empty());
    assert_eq!(
        std::fs::read_to_string(source.join("pack_info.dynx")).unwrap(),
        "PackName: Test\n"
    );
}
