use cpm_core::{Document, ObjMesh, ObjRequest, Pack};
use std::{fs, io::Write, path::Path};
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
