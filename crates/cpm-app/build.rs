fn main() {
    println!("cargo:rerun-if-changed=assets/cpm.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/cpm.ico")
            .set("ProductName", "CPM")
            .set("FileDescription", "CPM — Éditeur de packs DynamX")
            .set("InternalName", "cpm")
            .set("OriginalFilename", "cpm.exe")
            .compile()
            .expect("impossible de compiler les ressources Windows de CPM");
    }
}
