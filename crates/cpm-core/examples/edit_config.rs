use cpm_core::{Document, PackInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Exemple synthétique : aucun fichier existant n'est écrasé.
    let mut document = Document::parse(
        "// Métadonnées de démonstration\nPackName: Demo\nDcFileVersion: 12.5.0\n",
    )?;
    let name = document
        .properties()
        .iter()
        .position(|property| property.section.is_none() && property.key == "PackName")
        .ok_or("PackName absent")?;
    document.set_value(name, "MonPack")?;
    let info = PackInfo::from_document(&document).map_err(|errors| {
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    eprintln!("Pack validé : {}", info.pack_name);
    document.write_to(std::io::stdout())?;
    Ok(())
}
