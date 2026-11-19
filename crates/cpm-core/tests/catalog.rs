use cpm_core::*;
use std::{collections::BTreeMap, path::Path};

#[test]
fn builtin_roundtrip_and_addon_override_survive_restart() {
    let temp = tempfile::tempdir().unwrap();
    let store = CatalogStore {
        directory: temp.path().join("properties"),
    };
    let loaded = store.load().unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert!(store.directory.join("dynamx.json").is_file());
    validate_catalog_file(&loaded.catalog.files[0].catalog).unwrap();
    let addon = NamedCatalogFile {
        filename: "my-addon.json".into(),
        catalog: CatalogFile {
            schema_version: 1,
            label: "Demo".into(),
            rules: vec![CatalogRule {
                file_pattern: "engine_*.dynx".into(),
                section_pattern: None,
                parent_pattern: None,
                parameters: vec![ParameterDefinition {
                    name: "Power".into(),
                    variable_type: VariableType::Integer,
                    required: true,
                    severity: Severity::Warning,
                    default_value: Some("42".into()),
                    allow_empty: false,
                    ..Default::default()
                }],
            }],
        },
    };
    store.save_file(&addon).unwrap();
    let catalog = store.load().unwrap().catalog;
    let doc = Document::parse("Power: wrong\nMaxRPM: 6000\nBraking: 100\n").unwrap();
    assert!(catalog
        .errors(Path::new("engine_demo.dynx"), &doc)
        .is_empty());
    let diagnostics = catalog.diagnostics(Path::new("engine_demo.dynx"), &doc);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Warning);
    assert_eq!(
        catalog
            .parameters(Path::new("engine_demo.dynx"), &doc, None)
            .into_iter()
            .find(|p| p.name == "Power")
            .unwrap()
            .default_value
            .as_deref(),
        Some("42")
    );
    std::fs::write(store.directory.join("broken.json"), "{broken").unwrap();
    let loaded = store.load().unwrap();
    assert_eq!(loaded.warnings.len(), 1);
    assert_eq!(loaded.catalog.files.len(), 2);
    let mut invalid = addon.clone();
    invalid.filename = "../escape.json".into();
    assert!(store.save_file(&invalid).is_err());
    invalid = addon;
    invalid.catalog.rules[0].parameters[0].default_value = Some("wrong".into());
    assert!(store.save_file(&invalid).is_err());
    assert!(store
        .load()
        .unwrap()
        .catalog
        .errors(Path::new("engine_demo.dynx"), &doc)
        .is_empty());
}

#[test]
fn effective_catalog_controls_export_and_does_not_insert_defaults() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("pack_info.dynx"), "PackName: Demo\n").unwrap();
    let pack = Pack::open(&source).unwrap();
    let options = ExportOptions {
        destination: temp.path().into(),
        output_name: "output".into(),
        format: ExportFormat::Folder,
        replace_existing: false,
        metadata: PackMetadata {
            name: "Demo".into(),
            version: "1".into(),
            description: String::new(),
        },
    };
    assert!(pack.export(&options, &BTreeMap::new()).is_err());
    let mut catalog = ParameterCatalog::builtin();
    let rule = &mut catalog.files[0].catalog.rules[0];
    rule.parameters
        .iter_mut()
        .find(|p| p.name == "DcFileVersion")
        .unwrap()
        .required = false;
    rule.parameters.push(ParameterDefinition {
        name: "AddonValue".into(),
        required: true,
        severity: Severity::Warning,
        default_value: Some("suggestion".into()),
        ..Default::default()
    });
    let output = pack
        .export_with_catalog(&options, &BTreeMap::new(), &catalog)
        .unwrap();
    let text = std::fs::read_to_string(output.join("pack_info.dynx")).unwrap();
    assert!(!text.contains("AddonValue"));
    assert!(!text.contains("DcFileVersion"));
}

#[test]
fn scoped_rules_and_optional_pitch_range() {
    let doc = Document::parse("Engine{\nInterior{\nStarting{\nSound: test\n}\n}\n}\n").unwrap();
    let catalog = ParameterCatalog::builtin();
    let defs = catalog.parameters(Path::new("sounds_demo.dynx"), &doc, Some(2));
    assert!(
        !defs
            .iter()
            .find(|p| p.name == "PitchRange")
            .unwrap()
            .required
    );
    assert!(catalog
        .parameters(Path::new("sounds_demo.dynx"), &doc, Some(1))
        .is_empty());
}

#[test]
fn descriptions_are_backward_compatible_editable_and_preserve_explicit_empty_text() {
    let temp = tempfile::tempdir().unwrap();
    let store = CatalogStore {
        directory: temp.path().join("properties"),
    };
    let loaded = store.load().unwrap();
    assert!(loaded.catalog.files[0]
        .catalog
        .rules
        .iter()
        .flat_map(|r| &r.parameters)
        .all(|p| !p.description.is_empty()));
    let path = store.directory.join("dynamx.json");
    let mut json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let rules = json["rules"].as_array_mut().unwrap();
    for rule in rules {
        for parameter in rule["parameters"].as_array_mut().unwrap() {
            parameter.as_object_mut().unwrap().remove("description");
        }
    }
    std::fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
    let mut loaded = store.load().unwrap();
    assert!(loaded.warnings.is_empty());
    let parameter = &mut loaded.catalog.files[0].catalog.rules[0].parameters[0];
    assert!(!parameter.description.is_empty());
    parameter.description = "Description personnelle\navec détails".into();
    store.save_file(&loaded.catalog.files[0]).unwrap();
    assert_eq!(
        store.load().unwrap().catalog.files[0].catalog.rules[0].parameters[0].description,
        "Description personnelle\navec détails"
    );
    loaded.catalog.files[0].catalog.rules[0].parameters[0].description = String::new();
    store.save_file(&loaded.catalog.files[0]).unwrap();
    assert!(
        store.load().unwrap().catalog.files[0].catalog.rules[0].parameters[0]
            .description
            .is_empty()
    );
    let legacy: ParameterDefinition =
        serde_json::from_str(r#"{"name":"CustomNumber","variable_type":"integer"}"#).unwrap();
    assert!(legacy.description.is_empty());
}
