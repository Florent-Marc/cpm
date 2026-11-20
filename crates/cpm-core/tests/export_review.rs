use cpm_core::*;
use std::{collections::BTreeMap, fs, path::Path};
fn fixture() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    fs::write(
        t.path().join("pack_info.dynx"),
        "PackName: Demo\nPackVersion: 1\nDcFileVersion: 12.5.0\n",
    )
    .unwrap();
    for name in ["engine_a.dynx", "engine_b.dynx"] {
        fs::write(t.path().join(name),"// keep\nPower: 100\nMaxRPM: 5000\nBraking: 10\nPoint_1{\nRPMPower: 0 0\n}\nPoint_1{\nRPMPower: 900 1\n}\n").unwrap();
    }
    fs::write(t.path().join("car.obj"), "original model").unwrap();
    t
}
#[test]
fn reviewed_export_includes_metadata_resources_removals_and_exports_the_frozen_copy() {
    let source = fixture();
    let destination = tempfile::tempdir().unwrap();
    let mut pack = Pack::open(source.path()).unwrap();
    pack.remove_configuration(Path::new("engine_b.dynx"))
        .unwrap();
    pack.add_bundle(
        vec![(
            "custom_new.dynx".into(),
            Document::parse("Note: hello\n").unwrap(),
        )],
        BTreeMap::from([("new.png".into(), vec![1u8, 2, 3])]),
    )
    .unwrap();
    let options = ExportOptions {
        destination: destination.path().into(),
        output_name: "out".into(),
        format: ExportFormat::Zip,
        replace_existing: false,
        metadata: PackMetadata {
            name: "Demo".into(),
            version: "2".into(),
            description: String::new(),
        },
    };
    let docs = BTreeMap::from([(
        "engine_a.dynx".into(),
        Document::parse("Power: 200\nMaxRPM: 5000\nBraking: 10\n").unwrap(),
    )]);
    let review = pack
        .review_export(&options, &docs, &ParameterCatalog::builtin(), false)
        .unwrap();
    assert!(!destination.path().join("out.zip").exists());
    for (name, kind) in [
        ("engine_b.dynx", ChangeKind::Removed),
        ("custom_new.dynx", ChangeKind::Added),
        ("new.png", ChangeKind::Added),
        ("engine_a.dynx", ChangeKind::Modified),
    ] {
        assert!(review
            .changes
            .iter()
            .any(|c| c.path == Path::new(name) && c.kind == kind));
    }
    let metadata = review
        .changes
        .iter()
        .find(|c| c.path == Path::new("pack_info.dynx"))
        .unwrap();
    assert!(metadata
        .parameters
        .iter()
        .any(|c| c.parameter.contains("PackVersion")
            && c.before.as_deref() == Some("1")
            && c.after.as_deref() == Some("2")));
    fs::write(
        source.path().join("car.obj"),
        "external mutation after comparison",
    )
    .unwrap();
    let output = review.export().unwrap();
    let saved = Pack::open(&output).unwrap();
    assert_eq!(
        saved.read_resource(Path::new("car.obj"), 1024).unwrap(),
        b"original model"
    );
    assert!(saved
        .read_config(
            saved
                .configs()
                .iter()
                .position(|p| p == Path::new("engine_a.dynx"))
                .unwrap()
        )
        .unwrap()
        .source()
        .contains("Power: 200"));
}
#[test]
fn parameter_comparison_distinguishes_duplicate_sections_and_comments() {
    let before = Document::parse("Part{\nA: 1\n}\nPart{\nA: 2\n}\n").unwrap();
    let after = Document::parse("Part{\nA: 1\n}\nPart{\nA: 3\n}\n").unwrap();
    let changes = parameter_changes(Some(&before), Some(&after));
    assert_eq!(changes.len(), 1);
    assert!(changes[0].parameter.contains("Part #2"));
    assert_eq!(changes[0].before.as_deref(), Some("2"));
}
