use cpm_core::{Document, ParameterCatalog};
use eframe::egui;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Pack,
    Vehicle,
    Engine,
    Wheel,
    Sound,
    Trailer,
    Boat,
    Helicopter,
    Block,
    Prop,
    Item,
    Armor,
    Custom,
}
impl Kind {
    const ALL: [Self; 13] = [
        Self::Pack,
        Self::Vehicle,
        Self::Engine,
        Self::Wheel,
        Self::Sound,
        Self::Trailer,
        Self::Boat,
        Self::Helicopter,
        Self::Block,
        Self::Prop,
        Self::Item,
        Self::Armor,
        Self::Custom,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Pack => "Pack",
            Self::Vehicle => "Véhicule",
            Self::Engine => "Moteur",
            Self::Wheel => "Roue",
            Self::Sound => "Sons",
            Self::Trailer => "Remorque",
            Self::Boat => "Bateau",
            Self::Helicopter => "Hélicoptère",
            Self::Block => "Bloc",
            Self::Prop => "Prop",
            Self::Item => "Item",
            Self::Armor => "Armure",
            Self::Custom => "Personnalisé / addon",
        }
    }
    fn prefix(self) -> &'static str {
        match self {
            Self::Pack => "pack",
            Self::Vehicle => "vehicle",
            Self::Engine => "engine",
            Self::Wheel => "wheel",
            Self::Sound => "sounds",
            Self::Trailer => "trailer",
            Self::Boat => "boat",
            Self::Helicopter => "helicopter",
            Self::Block => "block",
            Self::Prop => "prop",
            Self::Item => "item",
            Self::Armor => "armor",
            Self::Custom => "",
        }
    }
}
pub enum Created {
    Pack { document: Document, name: String },
    Configuration { path: PathBuf, document: Document },
}
pub enum Action {
    None,
    Close,
    Create(Created),
}
pub struct Creation {
    kind: Kind,
    name: String,
    folder: String,
    pack_name: String,
    version: String,
    description: String,
    file_version: String,
    fields: Vec<(String, String)>,
    loaded_path: Option<PathBuf>,
    optional: bool,
    pub error: Option<String>,
}
impl Creation {
    pub fn new(in_pack: bool) -> Self {
        Self {
            kind: if in_pack { Kind::Vehicle } else { Kind::Pack },
            name: "nouveau".into(),
            folder: String::new(),
            pack_name: "MonPack".into(),
            version: "1.0".into(),
            description: String::new(),
            file_version: "12.5.0".into(),
            fields: Vec::new(),
            loaded_path: None,
            optional: false,
            error: None,
        }
    }
    fn path(&self) -> Result<PathBuf, String> {
        let name = self.name.trim();
        if name.is_empty()
            || name.contains(['/', '\\', ':'])
            || name
                .chars()
                .any(|c| c.is_control() || "*?\"<>|".contains(c))
            || name.ends_with(['.', ' '])
        {
            return Err("Saisir un nom de fichier valide, sans chemin".into());
        }
        let name = name.strip_suffix(".dynx").unwrap_or(name);
        if name.is_empty() || name == "." || name == ".." {
            return Err("Saisir un nom de fichier".into());
        }
        let filename = if self.kind == Kind::Custom {
            format!("{name}.dynx")
        } else {
            format!("{}_{name}.dynx", self.kind.prefix())
        };
        let folder = self.folder.trim().replace('\\', "/");
        if folder.starts_with('/') || folder.contains(':') || folder.split('/').any(|p| p == "..") {
            return Err("Le sous-dossier doit rester dans le pack".into());
        }
        let path = PathBuf::from(folder).join(filename);
        if path == Path::new("pack_info.dynx") {
            return Err("Utilise le type Pack pour créer un pack".into());
        }
        cpm_core::configuration_path(&path)
    }
    fn refresh(&mut self, catalog: &ParameterCatalog, path: &Path) {
        if self.loaded_path.as_deref() == Some(path) {
            return;
        }
        let document = Document::parse("").expect("vide");
        let old: std::collections::BTreeMap<_, _> = self.fields.drain(..).collect();
        self.fields = catalog
            .parameters(path, &document, None)
            .into_iter()
            .filter(|p| p.required || (self.optional && p.default_value.is_some()))
            .map(|p| {
                let value = old.get(&p.name).cloned().unwrap_or_else(|| {
                    if p.name == "Name" {
                        self.name.clone()
                    } else {
                        p.default_value.unwrap_or_default()
                    }
                });
                (p.name, value)
            })
            .collect();
        self.loaded_path = Some(path.to_owned());
    }
    fn build(&self) -> Result<Created, String> {
        let mut document = Document::parse("").map_err(|e| e.to_string())?;
        if self.kind == Kind::Pack {
            if self.pack_name.trim().is_empty() || self.file_version.trim().is_empty() {
                return Err("Renseigner le nom du pack et DcFileVersion".into());
            }
            for (key, value) in [
                ("PackName", &self.pack_name),
                ("PackVersion", &self.version),
                ("DcFileVersion", &self.file_version),
                ("Description", &self.description),
            ] {
                document
                    .add_property(None, key, value)
                    .map_err(|e| e.to_string())?;
            }
            Ok(Created::Pack {
                document,
                name: self.pack_name.trim().into(),
            })
        } else {
            let path = self.path()?;
            for (key, value) in &self.fields {
                document
                    .add_property(None, key, value)
                    .map_err(|e| e.to_string())?;
            }
            Ok(Created::Configuration { path, document })
        }
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        catalog: &ParameterCatalog,
        in_pack: bool,
        can_replace: bool,
    ) -> Action {
        let mut open = true;
        let mut action = Action::None;
        egui::Window::new("Créer").open(&mut open).collapsible(false).default_width(580.0).vscroll(true).show(ctx,|ui|{
            ui.horizontal(|ui|{ui.label("Type");egui::ComboBox::from_id_salt("creation-kind").selected_text(self.kind.label()).show_ui(ui,|ui|{for kind in Kind::ALL {if ui.selectable_value(&mut self.kind,kind,kind.label()).changed(){self.fields.clear();self.loaded_path=None;self.error=None;}}});});
            if self.kind==Kind::Pack {
                ui.label("Un nouveau pack sera préparé en mémoire. Utilise File → Exporter pour choisir sa destination et son format.");
                for (label,value) in [("Nom du pack",&mut self.pack_name),("Version",&mut self.version),("DcFileVersion",&mut self.file_version),("Description",&mut self.description)]{ui.horizontal(|ui|{ui.label(label);ui.text_edit_singleline(value);});}
            } else {
                ui.label(if in_pack{"La configuration sera ajoutée au pack ouvert."}else{"Une configuration seule sera créée. File → Exporter permettra de l’enregistrer."});
                ui.horizontal(|ui|{ui.label("Nom");ui.text_edit_singleline(&mut self.name);});
                if in_pack{ui.horizontal(|ui|{ui.label("Sous-dossier (facultatif)");ui.text_edit_singleline(&mut self.folder);});}
                if ui.checkbox(&mut self.optional,"Inclure les paramètres facultatifs avec une valeur par défaut").changed(){self.loaded_path=None;}
                match self.path() {
                    Ok(path)=>{
                        self.refresh(catalog,&path);ui.monospace(path.display().to_string());ui.separator();
                        let definitions=catalog.parameters(&path,&Document::parse("").expect("vide"),None);
                        for (key,value) in &mut self.fields{let definition=definitions.iter().find(|d|d.name==*key);ui.horizontal(|ui|{ui.label(key.as_str()).on_hover_text(definition.map(|d|d.description.as_str()).unwrap_or(""));crate::parameter_input::input(ui,value,definition,None);});}
                        ui.weak("Les paramètres obligatoires peuvent être complétés ensuite ; les erreurs bloqueront l’export.");
                    },Err(error)=>{ui.colored_label(egui::Color32::LIGHT_RED,error);}
                }
            }
            let allowed=(self.kind!=Kind::Pack && in_pack)||can_replace;
            if !allowed{ui.colored_label(egui::Color32::YELLOW,"Enregistre ou annule les modifications avant de créer un autre document ou pack.");}
            if let Some(error)=&self.error{ui.colored_label(egui::Color32::LIGHT_RED,error);}
            ui.horizontal(|ui|{
                if ui.add_enabled(allowed,egui::Button::new("Créer")).clicked(){match self.build(){Ok(created)=>action=Action::Create(created),Err(error)=>self.error=Some(error)}}
                if ui.button("Annuler").clicked(){action=Action::Close;}
            });
        });
        if !open {
            Action::Close
        } else {
            action
        }
    }
}

pub enum ManageAction {
    None,
    Close,
    Add,
    Delete(PathBuf),
    Import(PathBuf),
}
#[derive(Default)]
pub struct Manager {
    confirm: Option<PathBuf>,
    pub error: Option<String>,
}
impl Manager {
    pub fn show(&mut self, ctx: &egui::Context, paths: &[PathBuf]) -> ManageAction {
        let mut open = true;
        let mut action = ManageAction::None;
        egui::Window::new("Gérer les configurations").open(&mut open).default_width(540.0).collapsible(false).show(ctx,|ui|{
            ui.label("Les ajouts et suppressions seront appliqués lors de l’export du pack.");
            if ui.button("+ Créer une configuration").clicked(){action=ManageAction::Add;}
            if ui.button("Importer un fichier .dynx").clicked(){
                if let Some(path)=rfd::FileDialog::new().add_filter("Configuration DynamX",&["dynx"]).pick_file(){action=ManageAction::Import(path);}
            }
            if let Some(error)=&self.error{ui.colored_label(egui::Color32::LIGHT_RED,error);}
            egui::ScrollArea::vertical().max_height(330.0).show(ui,|ui|{for path in paths{ui.push_id(path,|ui|{ui.horizontal(|ui|{
                ui.label(crate::navigation::config_label(path)).on_hover_text(path.display().to_string());
                if path!=Path::new("pack_info.dynx") && ui.small_button("Supprimer").clicked(){self.confirm=Some(path.clone());}
            });});}});
            if let Some(path)=self.confirm.clone(){
                ui.separator();ui.colored_label(egui::Color32::YELLOW,format!("Supprimer {} de ce pack ? Ses modifications en mémoire seront abandonnées. Les références dans les autres fichiers ne seront pas modifiées.",path.display()));
                ui.horizontal(|ui|{if ui.button("Confirmer la suppression").clicked(){action=ManageAction::Delete(path);self.confirm=None;}
                    if ui.button("Conserver").clicked(){self.confirm=None;}});
            }
        });
        if !open {
            ManageAction::Close
        } else {
            action
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creates_metadata_and_configuration_fields_from_the_active_catalogue() {
        let mut form = Creation::new(false);
        let Created::Pack { document, .. } = form.build().unwrap() else {
            panic!()
        };
        assert!(cpm_core::ParameterCatalog::builtin()
            .errors(Path::new("pack_info.dynx"), &document)
            .is_empty());
        form.kind = Kind::Engine;
        form.name = "demo".into();
        form.folder = "engines".into();
        let path = form.path().unwrap();
        assert_eq!(path, Path::new("engines/engine_demo.dynx"));
        let mut catalog = ParameterCatalog::builtin();
        catalog.files[0]
            .catalog
            .rules
            .iter_mut()
            .find(|r| r.file_pattern == "engine_*.dynx" && r.section_pattern.is_none())
            .unwrap()
            .parameters
            .iter_mut()
            .find(|p| p.name == "Power")
            .unwrap()
            .default_value = Some("5000".into());
        form.refresh(&catalog, &path);
        assert!(form
            .fields
            .iter()
            .any(|(key, value)| key == "Power" && value == "5000"));
        let Created::Configuration { path, document } = form.build().unwrap() else {
            panic!()
        };
        assert!(document.source().contains("Power: 5000"));
        assert_eq!(path, Path::new("engines/engine_demo.dynx"));
        form.folder = "../../escape".into();
        assert!(form.build().is_err());
        form.folder = String::new();
        form.name = ".dynx".into();
        assert!(form.build().is_err());
    }
}
