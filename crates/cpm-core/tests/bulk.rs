use cpm_core::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
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
fn bulk_planning_is_atomic_typed_and_scoped_to_selected_files() {
    let t = fixture();
    let pack = Pack::open(t.path()).unwrap();
    let mut req = BulkRequest {
        paths: BTreeSet::from(["engine_a.dynx".into()]),
        parameter: "Power".into(),
        value: "300".into(),
        ..Default::default()
    };
    let plan = pack
        .plan_bulk_edit(&req, &BTreeMap::new(), &ParameterCatalog::builtin())
        .unwrap();
    assert_eq!(plan.documents.len(), 1);
    assert!(plan.documents[0].after.source().contains("Power: 300"));
    assert!(plan.documents[0].after.source().contains("// keep"));
    assert!(fs::read_to_string(t.path().join("engine_a.dynx"))
        .unwrap()
        .contains("Power: 100"));
    req.paths.insert("engine_b.dynx".into());
    req.value = "invalid number".into();
    assert!(pack
        .plan_bulk_edit(&req, &BTreeMap::new(), &ParameterCatalog::builtin())
        .is_err());
    req.parameter = "Custom".into();
    req.value = "abc".into();
    req.operation = BulkOperation::AddMissing;
    let plan = pack
        .plan_bulk_edit(&req, &BTreeMap::new(), &ParameterCatalog::builtin())
        .unwrap();
    assert_eq!(plan.documents.len(), 2);
    req.parameter = "RPMPower".into();
    req.section = Some("Point_1".into());
    req.value = "0 0.5".into();
    req.operation = BulkOperation::ReplaceExisting;
    let plan = pack
        .plan_bulk_edit(&req, &BTreeMap::new(), &ParameterCatalog::builtin())
        .unwrap();
    assert_eq!(
        plan.documents[0]
            .after
            .source()
            .matches("RPMPower: 0 0.5")
            .count(),
        2
    );
    req.operation = BulkOperation::Remove;
    let plan = pack
        .plan_bulk_edit(&req, &BTreeMap::new(), &ParameterCatalog::builtin())
        .unwrap();
    assert!(!plan.documents[0].after.source().contains("RPMPower:"));
}
