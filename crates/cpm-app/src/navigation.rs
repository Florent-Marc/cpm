use cpm_core::Section;
use std::path::{Path, PathBuf};

/// Libellé utilisateur du panneau ; le chemin reste l’identité de la configuration.
pub fn config_label(path: &Path) -> String {
    let name = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("dynx"))
    {
        path.file_stem().unwrap_or_default().to_string_lossy()
    } else {
        path.file_name().unwrap_or_default().to_string_lossy()
    };
    if name.eq_ignore_ascii_case("pack_info") {
        return "Informations du pack".into();
    }
    if let Some((prefix, label)) = name.split_once('_') {
        if !label.is_empty()
            && matches!(
                prefix.to_ascii_lowercase().as_str(),
                "vehicle"
                    | "engine"
                    | "sound"
                    | "sounds"
                    | "wheel"
                    | "trailer"
                    | "boat"
                    | "helicopter"
                    | "block"
                    | "prop"
                    | "item"
                    | "armor"
            )
        {
            return label.to_owned();
        }
    }
    name.into_owned()
}

pub fn config_groups(paths: &[PathBuf]) -> Vec<(&'static str, Vec<usize>)> {
    let categories = [
        "Pack",
        "Véhicules",
        "Moteurs",
        "Sons",
        "Roues",
        "Remorques",
        "Bateaux",
        "Hélicoptères",
        "Blocs",
        "Props",
        "Items",
        "Armures",
        "Autres",
    ];
    categories
        .into_iter()
        .filter_map(|category| {
            let indices: Vec<_> = paths
                .iter()
                .enumerate()
                .filter_map(|(index, path)| {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase();
                    let kind = if name == "pack_info.dynx" {
                        "Pack"
                    } else {
                        match name.split('_').next().unwrap_or("") {
                            "vehicle" => "Véhicules",
                            "engine" => "Moteurs",
                            "sound" | "sounds" => "Sons",
                            "wheel" => "Roues",
                            "trailer" => "Remorques",
                            "boat" => "Bateaux",
                            "helicopter" => "Hélicoptères",
                            "block" => "Blocs",
                            "prop" => "Props",
                            "item" => "Items",
                            "armor" => "Armures",
                            _ => "Autres",
                        }
                    };
                    (kind == category).then_some(index)
                })
                .collect();
            (!indices.is_empty()).then_some((category, indices))
        })
        .collect()
}

/// Groupement visuel seulement : les identités et l'ordre du fichier restent intacts.
pub fn section_groups(sections: &[Section]) -> Vec<(String, Vec<usize>)> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        let name = section.name.as_str();
        let key = ["Point", "Gear", "Shape", "Wheel", "Seat"]
            .into_iter()
            .find(|prefix| name.starts_with(prefix))
            .or_else(|| name.ends_with("Seat").then_some("Seat"))
            .unwrap_or_else(|| name.split_once('_').map_or(name, |(prefix, _)| prefix));
        if let Some((_, indices)) = groups.iter_mut().find(|(group, _)| group == key) {
            indices.push(index);
        } else {
            groups.push((key.to_owned(), vec![index]));
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configs_use_basename_and_keep_unknown_types_and_duplicate_names() {
        let paths = [
            "nested/engine_car.dynx",
            "other/engine_car.dynx",
            "pack_info.dynx",
            "sounds_car.dynx",
            "vehicle_car.dynx",
            "addon_custom.dynx",
        ]
        .map(PathBuf::from);
        let groups = config_groups(&paths);
        assert_eq!(
            groups,
            vec![
                ("Pack", vec![2]),
                ("Véhicules", vec![4]),
                ("Moteurs", vec![0, 1]),
                ("Sons", vec![3]),
                ("Autres", vec![5])
            ]
        );
    }
    #[test]
    fn section_families_keep_distinct_indices_and_source_order() {
        let document = cpm_core::Document::parse(
            "Point_1{\n}\nGear_0{\n}\nPoint_2{\n}\nDriverSeat{\n}\nPassengerSeat{\n}\nCustom{\n}\n",
        )
        .unwrap();
        assert_eq!(
            section_groups(document.sections()),
            vec![
                ("Point".into(), vec![0, 2]),
                ("Gear".into(), vec![1]),
                ("Seat".into(), vec![3, 4]),
                ("Custom".into(), vec![5])
            ]
        );
    }
}
