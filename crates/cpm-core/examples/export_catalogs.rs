//! Export bundled catalogues for release packaging (without launching the UI).
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::env::args_os()
        .nth(1)
        .ok_or("Dossier de sortie manquant")?;
    let store = cpm_core::CatalogStore {
        directory: destination.into(),
    };
    store.load()?;
    Ok(())
}
