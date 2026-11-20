use cpm_core::*;
use std::{fs, io::Write, path::Path};
fn zip(path: &Path, entries: &[(&str, &[u8])]) {
    let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
    for (name, bytes) in entries {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}
#[test]
fn recovery_uses_fresh_directories_and_rejects_traversal_duplicates_and_scripts() {
    let parent = tempfile::tempdir().unwrap();
    let archive = parent.path().join("source.zip");
    let info = b"PackName: Demo\nDcFileVersion: 12.5.0\n";
    zip(
        &archive,
        &[("pack_info.dynx", info), ("models/car.obj", b"model")],
    );
    let a = recover_pack_archive(&archive, parent.path(), "demo", "1").unwrap();
    let b = recover_pack_archive(&archive, parent.path(), "demo", "1").unwrap();
    assert_ne!(a, b);
    assert_eq!(fs::read(a.join("models/car.obj")).unwrap(), b"model");
    for bad in ["../escape.txt", "evil.php", "PACK_INFO.dynx"] {
        zip(&archive, &[("pack_info.dynx", info), (bad, b"bad")]);
        let before = fs::read_dir(parent.path()).unwrap().count();
        assert!(recover_pack_archive(&archive, parent.path(), "demo", "1").is_err());
        assert_eq!(before, fs::read_dir(parent.path()).unwrap().count());
    }
    assert!(!parent.path().join("escape.txt").exists());
}
