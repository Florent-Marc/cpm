use cpm_core::Pack;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "cpm-pack-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn zip(&self, name: &str, entries: &[(&str, &str)]) -> PathBuf {
        let path = self.0.join(name);
        let mut zip = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        for (name, content) in entries {
            zip.start_file(
                *name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn directory_is_recursive_sorted_and_only_loads_selected_config() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.0.join("car")).unwrap();
    fs::write(
        fixture.0.join("pack_info.dynx"),
        "PackName: Demo\nDcFileVersion: 12.5.0\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("car/vehicle_demo.dynx"),
        "Model: dynamxmod:car.obj\n",
    )
    .unwrap();
    fs::write(fixture.0.join("broken.dynx"), "not valid").unwrap();
    fs::write(fixture.0.join("texture.png"), [0xff]).unwrap();
    let pack = Pack::open(&fixture.0).unwrap();
    assert_eq!(
        pack.configs(),
        &[
            PathBuf::from("pack_info.dynx"),
            PathBuf::from("broken.dynx"),
            PathBuf::from("car/vehicle_demo.dynx")
        ]
    );
    assert_eq!(pack.read_config(0).unwrap().properties()[0].value, "Demo");
    assert!(pack.read_config(1).unwrap_err().contains("broken.dynx"));
    assert_eq!(
        pack.read_config(2).unwrap().properties()[0].value,
        "dynamxmod:car.obj"
    );
    assert!(pack.read_config(5).is_err());
}

#[test]
fn archive_reads_nested_configs_without_extracting_assets() {
    let fixture = Fixture::new();
    let path = fixture.zip(
        "demo.dnxpack",
        &[
            ("pack_info.dynx", "PackName: Archive\n"),
            ("car/vehicle_demo.dynx", "Name: Car\n"),
            ("assets/model.obj", "asset"),
        ],
    );
    let pack = Pack::open(&path).unwrap();
    assert_eq!(pack.configs().len(), 2);
    assert_eq!(pack.read_config(1).unwrap().properties()[0].value, "Car");
    assert!(!fixture.0.join("car").exists());
    assert!(!fixture.0.join("assets").exists());
}

#[test]
fn rejects_missing_metadata_and_unsafe_archive_paths() {
    let fixture = Fixture::new();
    assert!(Pack::open(&fixture.0)
        .unwrap_err()
        .contains("pack_info.dynx"));
    let path = fixture.zip(
        "unsafe.dnxpack",
        &[
            ("pack_info.dynx", "PackName: Test"),
            ("../outside.dynx", "Name: outside"),
        ],
    );
    assert!(Pack::open(&path).is_err());
}

#[test]
fn oversized_config_is_refused_in_folder_and_archive() {
    let fixture = Fixture::new();
    let huge = format!("Name: {}", "x".repeat(8 * 1024 * 1024));
    fs::write(fixture.0.join("pack_info.dynx"), &huge).unwrap();
    assert!(Pack::open(&fixture.0)
        .unwrap()
        .read_config(0)
        .unwrap_err()
        .contains("8 Mio"));
    let path = fixture.zip("large.dnxpack", &[("pack_info.dynx", &huge)]);
    assert!(Pack::open(&path)
        .unwrap()
        .read_config(0)
        .unwrap_err()
        .contains("8 Mio"));
}

#[cfg(unix)]
#[test]
fn directory_links_are_not_followed() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("pack_info.dynx"), "PackName: Test").unwrap();
    std::os::unix::fs::symlink(&fixture.0, fixture.0.join("cycle")).unwrap();
    let pack = Pack::open(&fixture.0).unwrap();
    assert_eq!(pack.configs().len(), 1);
}
