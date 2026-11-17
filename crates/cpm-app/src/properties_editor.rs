use cpm_core::{
    CatalogFile, CatalogRule, CatalogStore, NamedCatalogFile, ParameterCatalog,
    ParameterDefinition, Severity, VariableType,
};
use eframe::egui;
use std::path::PathBuf;

pub enum Action {
    None,
    Close,
    Applied(ParameterCatalog, Vec<String>),
}
pub struct PropertiesEditor {
    directory: PathBuf,
    files: Vec<NamedCatalogFile>,
    file: usize,
    rule: usize,
    parameter: usize,
    new_filename: String,
    pub error: Option<String>,
    choices_text: String,
    aliases_text: String,
    fields_for: Option<(usize, usize, usize)>,
}
impl PropertiesEditor {
    pub fn new(directory: PathBuf, catalog: &ParameterCatalog) -> Self {
        Self {
            directory,
            files: catalog.files.clone(),
            file: 0,
            rule: 0,
            parameter: 0,
            new_filename: "addon.json".into(),
            error: None,
            choices_text: String::new(),
            aliases_text: String::new(),
            fields_for: None,
        }
    }
    fn save(&mut self) -> Result<(ParameterCatalog, Vec<String>), String> {
        for file in &self.files {
            cpm_core::validate_catalog_file(&file.catalog)
                .map_err(|e| format!("{} : {e}", file.filename))?;
        }
        let store = CatalogStore {
            directory: self.directory.clone(),
        };
        let mut saved = Vec::new();
        for file in &self.files {
            store.save_file(file).map_err(|e| {
                format!(
                    "{} : {e}. Fichiers déjà enregistrés : {}",
                    file.filename,
                    saved.join(", ")
                )
            })?;
            saved.push(file.filename.clone());
        }
        let loaded = store.load()?;
        Ok((loaded.catalog, loaded.warnings))
    }
    pub fn show(&mut self, ctx: &egui::Context) -> Action {
        let mut action = Action::None;
        let mut open = true;
        egui::Window::new("Propriétés — Catalogue DynamX et addons").open(&mut open).default_width(940.0).default_height(640.0).resizable(true)
            .vscroll(true).collapsible(false).show(ctx,|ui|{
            ui.label("Dossier des définitions (à côté du programme)");
            ui.monospace(self.directory.display().to_string());
            ui.horizontal_wrapped(|ui|{
                egui::ComboBox::from_id_salt("properties-file").selected_text(self.files.get(self.file).map(|f|f.filename.as_str()).unwrap_or("Aucun fichier")).show_ui(ui,|ui|{
                    for (index,file) in self.files.iter().enumerate(){if ui.selectable_value(&mut self.file,index,&file.filename).changed(){self.rule=0;self.parameter=0;self.fields_for=None;}}
                });
                if ui.button("Recharger le dossier").on_hover_text("Annule les changements non enregistrés et recharge tous les JSON du dossier").clicked(){
                    match (CatalogStore{directory:self.directory.clone()}).load(){
                        Ok(loaded)=>{self.files=loaded.catalog.files;self.file=0;self.rule=0;self.parameter=0;self.fields_for=None;self.error=(!loaded.warnings.is_empty()).then(||loaded.warnings.join("\n"));}
                        Err(error)=>self.error=Some(error),
                    }
                }
                if ui.button("Importer JSON addon…").clicked(){
                    if let Some(path)=rfd::FileDialog::new().add_filter("Propriétés CPM",&["json"]).pick_file(){
                        let result=(||{
                            let catalog:CatalogFile=serde_json::from_slice(&std::fs::read(&path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                            cpm_core::validate_catalog_file(&catalog)?;
                            let filename=path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if self.files.iter().any(|f|f.filename.eq_ignore_ascii_case(&filename)){return Err("Un fichier portant ce nom est déjà chargé".into());}
                            Ok(NamedCatalogFile{filename,catalog})
                        })();
                        match result {Ok(file)=>{self.files.push(file);self.file=self.files.len()-1;self.rule=0;self.parameter=0;self.fields_for=None;self.error=None;},Err(error)=>self.error=Some(error)}
                    }
                }
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Nouveau fichier addon");ui.text_edit_singleline(&mut self.new_filename);
                if ui.button("Créer").clicked(){
                    if self.files.iter().any(|f|f.filename.eq_ignore_ascii_case(&self.new_filename)){self.error=Some("Ce nom de fichier existe déjà".into());}
                    else if self.new_filename.contains(['/', '\\', ':'])||!self.new_filename.ends_with(".json")||self.new_filename.starts_with('.') {self.error=Some("Saisir un nom de fichier .json sans chemin".into());}
                    else{self.files.push(NamedCatalogFile{filename:self.new_filename.clone(),catalog:CatalogFile{schema_version:1,label:"Addon".into(),rules:vec![CatalogRule::default()]}});self.file=self.files.len()-1;self.rule=0;self.parameter=0;self.fields_for=None;self.error=None;}
                }
            });
            ui.separator();
            if let Some(file)=self.files.get_mut(self.file){
                ui.horizontal(|ui|{ui.label("Nom du catalogue");ui.text_edit_singleline(&mut file.catalog.label);});
                ui.horizontal_wrapped(|ui|{
                    egui::ComboBox::from_id_salt("properties-rule").width(390.0).selected_text(file.catalog.rules.get(self.rule).map(rule_label).unwrap_or_else(||"Aucune règle".into())).show_ui(ui,|ui|{
                        for (index,rule) in file.catalog.rules.iter().enumerate(){if ui.selectable_value(&mut self.rule,index,rule_label(rule)).changed(){self.parameter=0;self.fields_for=None;}}
                    });
                    if ui.button("+ Règle").clicked(){file.catalog.rules.push(CatalogRule::default());self.rule=file.catalog.rules.len()-1;self.parameter=0;self.fields_for=None;}
                    if ui.add_enabled(!file.catalog.rules.is_empty(),egui::Button::new("− Règle")).clicked(){file.catalog.rules.remove(self.rule);self.rule=0;self.parameter=0;self.fields_for=None;}
                });
                if let Some(rule)=file.catalog.rules.get_mut(self.rule){
                    ui.horizontal_wrapped(|ui|{
                        ui.label("Fichiers");ui.text_edit_singleline(&mut rule.file_pattern).on_hover_text("Nom ou motif de fichier, par exemple vehicle_*.dynx ou machine_*.dynx");
                        let mut root=rule.section_pattern.is_none();
                        if ui.checkbox(&mut root,"Racine").changed(){rule.section_pattern=(!root).then(||"*".into());if root{rule.parent_pattern=None;}}
                    });
                    if let Some(pattern)=&mut rule.section_pattern{
                        ui.horizontal_wrapped(|ui|{ui.label("Section");ui.text_edit_singleline(pattern);let mut parent=rule.parent_pattern.is_some();if ui.checkbox(&mut parent,"Limiter au parent").changed(){rule.parent_pattern=parent.then(String::new);}});
                        if let Some(parent)=&mut rule.parent_pattern{ui.horizontal(|ui|{ui.label("Parent");ui.text_edit_singleline(parent);});}
                    }
                    ui.horizontal(|ui|{
                        if ui.button("+ Paramètre").clicked(){let mut name="NewParameter".to_owned();let mut suffix=2;while rule.parameters.iter().any(|p|p.name==name){name=format!("NewParameter{suffix}");suffix+=1;}rule.parameters.push(ParameterDefinition{name,..Default::default()});self.parameter=rule.parameters.len()-1;self.fields_for=None;}
                        if ui.add_enabled(!rule.parameters.is_empty(),egui::Button::new("− Définition")).on_hover_text("Retire cette définition du catalogue sans supprimer de valeurs dans les packs").clicked(){rule.parameters.remove(self.parameter);self.parameter=0;self.fields_for=None;}
                    });
                    ui.columns(2,|columns|{
                        egui::ScrollArea::vertical().id_salt("properties-list").max_height(290.0).show(&mut columns[0],|ui|{
                            for (index,parameter) in rule.parameters.iter().enumerate(){
                                let problem=definition_problem(rule,index);
                                let mut label=egui::RichText::new(&parameter.name);
                                if problem.is_some(){label=label.color(egui::Color32::LIGHT_RED);}
                                if ui.selectable_value(&mut self.parameter,index,label).on_hover_text(problem.unwrap_or_default()).changed(){self.fields_for=None;}
                            }
                        });
                        let problem=definition_problem(rule,self.parameter);
                        if let Some(parameter)=rule.parameters.get_mut(self.parameter){
                            let ui=&mut columns[1];
                            if let Some(message)=&problem{ui.colored_label(egui::Color32::LIGHT_RED,message);}
                            ui.horizontal(|ui|{ui.label("Paramètre");ui.text_edit_singleline(&mut parameter.name);});
                            egui::ComboBox::from_id_salt("properties-type").selected_text(parameter.variable_type.label()).show_ui(ui,|ui|{for kind in VariableType::ALL{ui.selectable_value(&mut parameter.variable_type,kind,kind.label());}});
                            ui.checkbox(&mut parameter.required,"Obligatoire");
                            ui.checkbox(&mut parameter.allow_empty,"Autoriser une valeur vide");
                            egui::ComboBox::from_id_salt("properties-severity").selected_text(parameter.severity.label()).show_ui(ui,|ui|{for severity in [Severity::Ignore,Severity::Warning,Severity::Error]{ui.selectable_value(&mut parameter.severity,severity,severity.label());}});
                            let mut has_default=parameter.default_value.is_some();
                            if ui.checkbox(&mut has_default,"Valeur par défaut").changed(){parameter.default_value=has_default.then(String::new);}
                            if let Some(value)=&mut parameter.default_value{ui.text_edit_singleline(value);}
                            let key=(self.file,self.rule,self.parameter);
                            if self.fields_for!=Some(key){self.choices_text=parameter.choices.join(", ");self.aliases_text=parameter.aliases.join(", ");self.fields_for=Some(key);}
                            if parameter.variable_type==VariableType::Enumeration{ui.label("Choix autorisés, séparés par des virgules");if ui.text_edit_singleline(&mut self.choices_text).changed(){parameter.choices=split_list(&self.choices_text);}}
                            ui.label("Alias (facultatif)");if ui.text_edit_singleline(&mut self.aliases_text).changed(){parameter.aliases=split_list(&self.aliases_text);}
                        }
                    });
                }
            }
            ui.separator();
            if let Some(error)=&self.error{ui.colored_label(egui::Color32::LIGHT_RED,error);}
            ui.horizontal_wrapped(|ui|{
                if ui.button("Enregistrer et appliquer").clicked(){match self.save(){Ok((catalog,warnings))=>action=Action::Applied(catalog,warnings),Err(error)=>self.error=Some(error)}}
                if ui.button("Fermer sans appliquer").clicked(){action=Action::Close;}
            });
        });
        if !open {
            action = Action::Close;
        }
        action
    }
}
fn definition_problem(rule: &CatalogRule, index: usize) -> Option<String> {
    let parameter = rule.parameters.get(index)?;
    for name in std::iter::once(&parameter.name).chain(parameter.aliases.iter()) {
        if rule
            .parameters
            .iter()
            .enumerate()
            .any(|(other, p)| other != index && (p.name == *name || p.aliases.contains(name)))
        {
            return Some(format!("Paramètre ou alias dupliqué : {name}"));
        }
    }
    let single = CatalogFile {
        schema_version: 1,
        label: String::new(),
        rules: vec![CatalogRule {
            file_pattern: "*.dynx".into(),
            section_pattern: None,
            parent_pattern: None,
            parameters: vec![parameter.clone()],
        }],
    };
    cpm_core::validate_catalog_file(&single).err()
}

fn split_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}
fn rule_label(rule: &CatalogRule) -> String {
    format!(
        "{} / {}{}",
        rule.file_pattern,
        rule.section_pattern.as_deref().unwrap_or("Racine"),
        rule.parent_pattern
            .as_ref()
            .map(|p| format!(" (parent {p})"))
            .unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifies_invalid_defaults_and_both_duplicate_definitions() {
        let mut rule = CatalogRule {
            parameters: vec![ParameterDefinition {
                name: "Capacity".into(),
                variable_type: VariableType::Integer,
                default_value: Some("wrong".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(definition_problem(&rule, 0).unwrap().contains("défaut"));
        rule.parameters[0].default_value = Some("42".into());
        assert!(definition_problem(&rule, 0).is_none());
        rule.parameters.push(ParameterDefinition {
            name: "Other".into(),
            aliases: vec!["Capacity".into()],
            ..Default::default()
        });
        assert!(definition_problem(&rule, 0).unwrap().contains("dupliqué"));
        assert!(definition_problem(&rule, 1).unwrap().contains("dupliqué"));
    }

    #[test]
    fn draft_changes_apply_only_after_save_and_reload() {
        let directory = std::env::temp_dir().join(format!("cpm-properties-{}", std::process::id()));
        let store = CatalogStore {
            directory: directory.clone(),
        };
        let original = store.load().unwrap().catalog;
        let mut window = PropertiesEditor::new(directory.clone(), &original);
        let definition = &mut window.files[0].catalog.rules[0].parameters[0];
        let name = definition.name.clone();
        definition.required = !definition.required;
        definition.severity = Severity::Warning;
        let required = definition.required;
        assert_ne!(
            store.load().unwrap().catalog.files[0].catalog.rules[0].parameters[0].required,
            required
        );
        let (applied, warnings) = window.save().unwrap();
        assert!(warnings.is_empty());
        assert_eq!(applied.files[0].catalog.rules[0].parameters[0].name, name);
        assert_eq!(
            applied.files[0].catalog.rules[0].parameters[0].required,
            required
        );
        assert_eq!(
            store.load().unwrap().catalog.files[0].catalog.rules[0].parameters[0].severity,
            Severity::Warning
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
