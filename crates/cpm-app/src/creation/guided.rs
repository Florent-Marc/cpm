use super::{Created, Creation, Kind};
use cpm_core::{Document, ModelFiles, Pack, ParameterCatalog};
use eframe::egui;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    Type,
    Assets,
    Properties,
    Assembly,
    Summary,
}
impl Step {
    pub fn title(self) -> &'static str {
        match self {
            Self::Type => "Votre projet",
            Self::Assets => "Modèle et textures",
            Self::Properties => "Propriétés",
            Self::Assembly => "Assemblage",
            Self::Summary => "Vérification et création",
        }
    }
}
#[derive(Clone, Default)]
pub(super) struct ModelChoice {
    pub existing: Option<PathBuf>,
    pub files: ModelFiles,
}
impl ModelChoice {
    pub fn path(&self, folder: &Path) -> Option<PathBuf> {
        let path = self
            .files
            .obj
            .as_ref()
            .map(|p| {
                folder.join(
                    if p.extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("gltf"))
                    {
                        "model.gltf"
                    } else {
                        "model.obj"
                    },
                )
            })
            .or_else(|| self.existing.clone());
        path.filter(|p| {
            !p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("gltf"))
                || self.files.dynamx_beta_confirmed
        })
    }
    pub fn prepare(
        &self,
        folder: &Path,
        assets: &mut BTreeMap<PathBuf, Vec<u8>>,
    ) -> Result<PathBuf, String> {
        if self.path(folder).is_none() {
            return Err("Choisir un modèle et confirmer DynamX bêta pour un glTF".into());
        }
        if self.files.obj.is_some() {
            let prepared = self.files.prepare(folder)?;
            for (p, b) in prepared.assets {
                if assets.insert(p, b).is_some() {
                    return Err("Deux modèles utilisent le même dossier de ressources".into());
                }
            }
            Ok(prepared.model)
        } else {
            self.existing
                .clone()
                .ok_or_else(|| "Choisir un modèle dans le pack ou importer un modèle".into())
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui, pack: Option<&Pack>, id: &str) {
        ui.push_id(id, |ui| {
            ui.colored_label(egui::Color32::YELLOW, "glTF : utilisez-vous bien la version bêta de DynamX ? Sans confirmation, l’ajout de modèles .gltf est désactivé. GLB non accepté.");
            ui.checkbox(&mut self.files.dynamx_beta_confirmed, "Oui, j’utilise DynamX bêta (autoriser le glTF)");
            let beta = self.files.dynamx_beta_confirmed;
            if let Some(pack) = pack {
                egui::ComboBox::from_id_salt("existing-model").selected_text(self.existing.as_ref().map(|p|p.display().to_string()).unwrap_or("Choisir un modèle déjà dans le pack…".into())).show_ui(ui,|ui|{
                    for path in pack.model_resources() {
                        let allowed = beta || !path.extension().is_some_and(|e|e.eq_ignore_ascii_case("gltf"));
                        if ui.add_enabled(allowed,egui::Button::selectable(self.existing.as_ref()==Some(&path),path.display().to_string())).clicked(){self.existing=Some(path);self.files=ModelFiles {dynamx_beta_confirmed:beta,..Default::default()};}
                    }
                });
            }
            if ui.button(if beta {"Choisir un fichier OBJ ou glTF…"}else{"Choisir un fichier OBJ…"}).clicked(){
                let extensions: &[&str] = if beta {&["obj","gltf"]}else{&["obj"]};
                if let Some(path)=rfd::FileDialog::new().add_filter("Modèle 3D",extensions).pick_file(){self.existing=None;self.files=ModelFiles {obj:Some(path),dynamx_beta_confirmed:beta,..Default::default()};}
            }
            if let Some(path)=&self.files.obj {ui.monospace(path.display().to_string());}
            if self.files.obj.is_some(){
                let gltf=self.files.obj.as_ref().is_some_and(|p|p.extension().is_some_and(|e|e.eq_ignore_ascii_case("gltf")));
                if gltf {
                    ui.label("Le glTF contient ses matériaux. Ajouter les buffers .bin et les images référencées ; les fichiers intégrés en base64 n’ont pas à être sélectionnés. Aperçu statique sans animations ni rendu PBR complet.");
                    if ui.button("Ajouter les buffers BIN…").clicked(){if let Some(paths)=rfd::FileDialog::new().add_filter("Buffers glTF",&["bin"]).pick_files(){for path in paths {if !self.files.materials.contains(&path){self.files.materials.push(path);}}}}
                    file_list(ui,&mut self.files.materials,"buffers");
                } else {
                    ui.checkbox(&mut self.files.untextured,"Importer sans matériaux ni textures");
                    if !self.files.untextured {
                        ui.label("Le MTL décrit les matériaux. Sélectionner tous les MTL et toutes les images de cet OBJ.");
                        if ui.button("Ajouter un ou plusieurs MTL…").clicked(){if let Some(paths)=rfd::FileDialog::new().add_filter("Matériaux MTL",&["mtl","mlt"]).pick_files(){for path in paths {if !self.files.materials.contains(&path){self.files.materials.push(path);}}}}
                        file_list(ui,&mut self.files.materials,"mtls");
                    }
                }
                if gltf || !self.files.untextured {
                    if ui.button("Ajouter des textures…").clicked(){if let Some(paths)=rfd::FileDialog::new().add_filter("Textures",&["png","jpg","jpeg","tga","bmp"]).pick_files(){for path in paths {if !self.files.textures.contains(&path){self.files.textures.push(path);}}}}
                    file_list(ui,&mut self.files.textures,"textures");
                }
            }
        });
    }
}
fn file_list(ui: &mut egui::Ui, files: &mut Vec<PathBuf>, id: &str) {
    let mut remove = None;
    ui.label(format!("{} fichier(s) sélectionné(s)", files.len()));
    egui::ScrollArea::vertical()
        .id_salt(id)
        .max_height(100.0)
        .show(ui, |ui| {
            for (i, path) in files.iter().enumerate() {
                ui.horizontal(|ui| {
                    if ui.small_button("×").clicked() {
                        remove = Some(i);
                    }
                    ui.label(path.display().to_string());
                });
            }
        });
    if let Some(i) = remove {
        files.remove(i);
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum WheelMode {
    #[default]
    Later,
    Existing,
    Import,
    New,
}
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum EngineMode {
    #[default]
    Later,
    Existing,
    New,
}
#[derive(Clone)]
pub(super) struct WheelRow {
    name: String,
    position: String,
    driving: bool,
    right: bool,
    steering: bool,
    handbrake: bool,
    max_turn: String,
    existing: Option<PathBuf>,
}
#[derive(Clone)]
pub(super) struct Guide {
    pub allow_incomplete: bool,
    pub step: usize,
    pub new_pack: bool,
    pub model: ModelChoice,
    pub wheel_mode: WheelMode,
    pub wheel_existing: Option<PathBuf>,
    pub wheel_import: Option<PathBuf>,
    pub wheel_name: String,
    pub wheel_model: ModelChoice,
    pub wheel_fields: Vec<(String, String)>,
    wheel_path: Option<PathBuf>,
    pub rows: Vec<WheelRow>,
    pub engine_mode: EngineMode,
    pub engine_existing: Option<PathBuf>,
    pub engine_name: String,
    pub engine_fields: Vec<(String, String)>,
    engine_path: Option<PathBuf>,
    pub sounds: Option<PathBuf>,
    pub seat: bool,
    pub seat_position: String,
    pub collision: bool,
    pub collision_position: String,
    pub collision_scale: String,
    pub steering: bool,
    pub steering_object: String,
    pub steering_position: String,
}
impl Default for Guide {
    fn default() -> Self {
        Self {
            allow_incomplete: false,
            step: 0,
            new_pack: false,
            model: ModelChoice::default(),
            wheel_mode: WheelMode::Later,
            wheel_existing: None,
            wheel_import: None,
            wheel_name: "roue".into(),
            wheel_model: ModelChoice::default(),
            wheel_fields: Vec::new(),
            wheel_path: None,
            rows: [
                ("front_left", "-0.8 1.4 0.35", false),
                ("front_right", "0.8 1.4 0.35", true),
                ("back_left", "-0.8 -1.4 0.35", false),
                ("back_right", "0.8 -1.4 0.35", true),
            ]
            .into_iter()
            .enumerate()
            .map(|(i, (n, p, r))| WheelRow {
                name: format!("Wheel_{n}"),
                position: p.into(),
                driving: i >= 2,
                right: r,
                steering: i < 2,
                handbrake: i >= 2,
                max_turn: "30".into(),
                existing: None,
            })
            .collect(),
            engine_mode: EngineMode::Later,
            engine_existing: None,
            engine_name: "moteur".into(),
            engine_fields: Vec::new(),
            engine_path: None,
            sounds: None,
            seat: true,
            seat_position: "0 0 0.5".into(),
            collision: false,
            collision_position: "0 0 0.8".into(),
            collision_scale: "0.8 1.8 0.7".into(),
            steering: false,
            steering_object: "SteeringWheel".into(),
            steering_position: String::new(),
        }
    }
}
impl Guide {
    pub fn steps(kind: Kind) -> Vec<Step> {
        let mut steps = vec![Step::Type];
        if kind.has_model() {
            steps.push(Step::Assets);
        }
        if kind != Kind::Pack {
            steps.push(Step::Properties);
        }
        if kind.has_assembly() {
            steps.push(Step::Assembly);
        }
        steps.push(Step::Summary);
        steps
    }
    pub fn assembly_ui(
        &mut self,
        ui: &mut egui::Ui,
        catalog: &ParameterCatalog,
        pack: Option<&Pack>,
        kind: Kind,
    ) {
        let paths = pack.map(Pack::configs).unwrap_or_default();
        if kind.has_wheels() {
            ui.heading("Roues");
            ui.label("Une configuration de roue décrit le modèle du pneu et sa suspension. Chaque emplacement du véhicule indique sa position et son rôle.");
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.wheel_mode, WheelMode::Later, "Plus tard");
                ui.selectable_value(&mut self.wheel_mode, WheelMode::Existing, "Roue du pack");
                ui.selectable_value(
                    &mut self.wheel_mode,
                    WheelMode::Import,
                    "Importer une configuration",
                );
                ui.selectable_value(&mut self.wheel_mode, WheelMode::New, "Créer une roue");
            });
            match self.wheel_mode {
                WheelMode::Later => {
                    ui.weak("Le véhicule sera créé sans roues. Vous pourrez les ajouter ensuite ; il ne sera pas prêt à rouler.");
                }
                WheelMode::Existing => {
                    configuration_combo(
                        ui,
                        "wheel-shared",
                        &mut self.wheel_existing,
                        paths,
                        "wheel_",
                        "Choisir une configuration de roue…",
                    );
                }
                WheelMode::Import | WheelMode::New => {
                    if self.wheel_mode == WheelMode::Import {
                        if ui
                            .button("Choisir une configuration de roue .dynx…")
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Configuration DynamX", &["dynx"])
                                .pick_file()
                            {
                                self.wheel_name = path
                                    .file_stem()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .trim_start_matches("wheel_")
                                    .to_owned();
                                self.wheel_import = Some(path);
                            }
                        }
                        if let Some(path) = &self.wheel_import {
                            ui.monospace(path.display().to_string());
                        }
                        ui.weak("Si son modèle est déjà dans le pack, il sera réutilisé. Sinon, sélectionnez aussi son OBJ, son MTL et ses textures ci-dessous.");
                    }
                    ui.horizontal(|ui| {
                        ui.label("Nom du fichier de roue");
                        ui.text_edit_singleline(&mut self.wheel_name);
                    });
                    if let Ok(path) = named_path("wheel", &self.wheel_name) {
                        refresh_fields(
                            &mut self.wheel_fields,
                            &mut self.wheel_path,
                            catalog,
                            &path,
                        );
                    }
                    self.wheel_model.ui(ui, pack, "wheel-model");
                    if self.wheel_mode == WheelMode::New {
                        ui.collapsing("Dimensions, freinage et suspension de la roue",|ui|{ui.weak("Valeurs de départ à adapter à votre modèle et à tester dans DynamX. Radius et Width décrivent la roue physique ; ScaleModifier règle la taille du modèle OBJ.");if let Ok(path)=named_path("wheel",&self.wheel_name){fields_ui(ui,&mut self.wheel_fields,catalog,&path,true);}});
                    }
                }
            }
            if self.wheel_mode != WheelMode::Later {
                ui.separator();
                ui.weak("Positions proposées pour démarrer : adaptez-les aux dimensions de votre OBJ. X = gauche/droite, Y = avant/arrière, Z = hauteur ; vous pourrez aussi utiliser les flèches 3D après création.");
                let mut remove = None;
                for (i, row) in self.rows.iter_mut().enumerate() {
                    ui.push_id(("wheel", i), |ui| {
                        egui::CollapsingHeader::new(&row.name)
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Nom de section");
                                    ui.text_edit_singleline(&mut row.name);
                                    if ui.small_button("Retirer").clicked() {
                                        remove = Some(i);
                                    }
                                });
                                vector_ui(ui, "Position", &mut row.position);
                                configuration_combo(
                                    ui,
                                    "wheel-override",
                                    &mut row.existing,
                                    paths,
                                    "wheel_",
                                    "Même roue que les autres",
                                );
                                ui.horizontal_wrapped(|ui| {
                                    ui.checkbox(&mut row.driving, "Motrice");
                                    ui.checkbox(&mut row.right, "Côté droit");
                                    ui.checkbox(&mut row.steering, "Directrice");
                                    ui.checkbox(&mut row.handbrake, "Frein à main");
                                });
                                if row.steering {
                                    ui.horizontal(|ui| {
                                        ui.label("Angle maximal de braquage");
                                        number_ui(ui, &mut row.max_turn);
                                    });
                                }
                            });
                    });
                }
                if let Some(i) = remove {
                    self.rows.remove(i);
                }
                if ui.button("+ Ajouter un emplacement de roue").clicked() {
                    let mut n = self.rows.len() + 1;
                    while self
                        .rows
                        .iter()
                        .any(|r| r.name == format!("Wheel_extra_{n}"))
                    {
                        n += 1;
                    }
                    self.rows.push(WheelRow {
                        name: format!("Wheel_extra_{n}"),
                        position: "0 0 0".into(),
                        driving: false,
                        right: false,
                        steering: false,
                        handbrake: false,
                        max_turn: "30".into(),
                        existing: None,
                    });
                }
            }
            ui.separator();
            ui.heading("Moteur");
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.engine_mode, EngineMode::Later, "Plus tard");
                ui.selectable_value(
                    &mut self.engine_mode,
                    EngineMode::Existing,
                    "Moteur du pack",
                );
                ui.selectable_value(
                    &mut self.engine_mode,
                    EngineMode::New,
                    "Créer un moteur de base",
                );
            });
            match self.engine_mode {
                EngineMode::Later => {
                    ui.weak("Sans moteur, ce véhicule ne sera pas prêt à rouler.");
                }
                EngineMode::Existing => configuration_combo(
                    ui,
                    "engine-shared",
                    &mut self.engine_existing,
                    paths,
                    "engine_",
                    "Choisir un moteur…",
                ),
                EngineMode::New => {
                    ui.horizontal(|ui| {
                        ui.label("Nom du fichier moteur");
                        ui.text_edit_singleline(&mut self.engine_name);
                    });
                    if let Ok(path) = named_path("engine", &self.engine_name) {
                        refresh_fields(
                            &mut self.engine_fields,
                            &mut self.engine_path,
                            catalog,
                            &path,
                        );
                        fields_ui(ui, &mut self.engine_fields, catalog, &path, false);
                    }
                    ui.weak("Une courbe et une boîte de vitesses de départ seront créées. Ajustez leurs valeurs dans la configuration du moteur, puis testez en jeu.");
                }
            }
            ui.label("Sons (facultatif)");
            configuration_combo(
                ui,
                "sounds-shared",
                &mut self.sounds,
                paths,
                "sounds_",
                "Configurer les sons plus tard",
            );
        }
        if kind.is_vehicle() {
            ui.separator();
            ui.checkbox(&mut self.seat, "Ajouter un siège conducteur");
            if self.seat {
                vector_ui(ui, "Position du conducteur", &mut self.seat_position);
                ui.weak("Placez le siège dans l’habitacle ; la position de départ doit être ajustée au modèle.");
            }
            ui.checkbox(&mut self.steering, "Déclarer une pièce de volant");
            if self.steering {
                ui.horizontal(|ui| {
                    ui.label("Nom de la pièce dans l’OBJ");
                    ui.text_edit_singleline(&mut self.steering_object);
                });
                ui.label(
                    "Laisser la Position vide pour conserver le placement du volant dans l’OBJ.",
                );
                vector_ui(
                    ui,
                    "Position manuelle (facultative)",
                    &mut self.steering_position,
                );
            }
        }
        if kind.has_collisions() {
            ui.separator();
            ui.checkbox(&mut self.collision, "Ajouter une boîte de collision simple");
            if self.collision {
                ui.label("La boîte sert de point de départ : ses dimensions sont des demi-dimensions. Ajustez-la ensuite dans l’aperçu 3D.");
                vector_ui(ui, "Centre de la boîte", &mut self.collision_position);
                vector_ui(ui, "Demi-dimensions", &mut self.collision_scale);
            }
        }
    }
}
pub(super) fn named_path(prefix: &str, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err("Pour les fichiers générés, utiliser des lettres, chiffres, tirets ou underscores, sans espace".into());
    }
    cpm_core::configuration_path(&PathBuf::from(format!("{prefix}_{name}.dynx")))
}
pub(super) fn folder(path: &Path) -> PathBuf {
    PathBuf::from("models").join(path.file_stem().unwrap_or_default())
}
fn configuration_combo(
    ui: &mut egui::Ui,
    id: &str,
    current: &mut Option<PathBuf>,
    paths: &[PathBuf],
    prefix: &str,
    none: &str,
) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(
            current
                .as_ref()
                .map(|p| crate::navigation::config_label(p))
                .unwrap_or(none.into()),
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(current, None, none);
            for path in paths.iter().filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().to_lowercase().starts_with(prefix))
            }) {
                ui.selectable_value(
                    current,
                    Some(path.clone()),
                    crate::navigation::config_label(path),
                );
            }
        });
}
pub(super) fn initial(key: &str) -> Option<&'static str> {
    Some(match key {
        "EmptyMass" => "1000",
        "DragCoefficient" => "0.3",
        "CenterOfGravityOffset" => "0 0 0",
        "Power" => "1000",
        "MaxRPM" => "6000",
        "Braking" => "100",
        "BrakeForce" => "1000",
        "Friction" => "1",
        "Radius" => "0.35",
        "RimRadius" => "0.25",
        "RollInInfluence" => "0.1",
        "SuspensionMaxForce" => "6000",
        "SuspensionRestLength" => "0.3",
        "SuspensionStiffness" => "20",
        "WheelDampingRelaxation" => "2.3",
        "WheelsDampingCompression" => "4.4",
        "Width" => "0.2",
        _ => return None,
    })
}
fn refresh_fields(
    fields: &mut Vec<(String, String)>,
    loaded: &mut Option<PathBuf>,
    catalog: &ParameterCatalog,
    path: &Path,
) {
    if loaded.as_deref() == Some(path) {
        return;
    }
    let old: BTreeMap<_, _> = fields.drain(..).collect();
    let empty = Document::parse("").unwrap();
    *fields = catalog
        .parameters(path, &empty, None)
        .into_iter()
        .filter(|p| p.required || p.name == "ScaleModifier")
        .map(|p| {
            let value = old
                .get(&p.name)
                .cloned()
                .or_else(|| p.default_value.clone())
                .or_else(|| initial(&p.name).map(str::to_owned))
                .unwrap_or_default();
            (p.name, value)
        })
        .collect();
    *loaded = Some(path.into());
}
pub(super) fn fields_ui(
    ui: &mut egui::Ui,
    fields: &mut [(String, String)],
    catalog: &ParameterCatalog,
    path: &Path,
    hide_model: bool,
) {
    let empty = Document::parse("").unwrap();
    let defs = catalog.parameters(path, &empty, None);
    let mut document = empty;
    for (key, value) in fields.iter() {
        let _ = document.add_property(None, key, value);
    }
    let diagnostics = catalog.diagnostics(path, &document);
    for (key, value) in fields
        .iter_mut()
        .filter(|(key, _)| !hide_model || key != "Model")
    {
        ui.push_id(key.clone(), |ui| {
            ui.horizontal(|ui| {
                let definition = defs.iter().find(|d| d.name == *key);
                let errors: Vec<_> = diagnostics
                    .iter()
                    .filter(|d| d.parameter == *key && d.severity == cpm_core::Severity::Error)
                    .map(ToString::to_string)
                    .collect();
                let color = (!errors.is_empty()).then_some(egui::Color32::LIGHT_RED);
                let text = egui::RichText::new(key.as_str());
                ui.label(if let Some(color) = color {
                    text.color(color)
                } else {
                    text
                })
                .on_hover_text(format!(
                    "{}\n{}",
                    definition
                        .map(|d| d.description.as_str())
                        .unwrap_or("Paramètre personnalisé"),
                    errors.join("\n")
                ));
                crate::parameter_input::input(ui, value, definition, color);
            });
        });
    }
}

fn number_ui(ui: &mut egui::Ui, value: &mut String) {
    crate::parameter_input::input(
        ui,
        value,
        Some(&cpm_core::ParameterDefinition {
            variable_type: cpm_core::VariableType::Decimal,
            ..Default::default()
        }),
        None,
    );
}
fn vector_ui(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        crate::parameter_input::input(
            ui,
            value,
            Some(&cpm_core::ParameterDefinition {
                variable_type: cpm_core::VariableType::Vector3,
                ..Default::default()
            }),
            None,
        );
    });
}
fn check_vector(text: &str, positive: bool) -> Result<(), String> {
    let values: Vec<_> = text
        .split_whitespace()
        .map(str::parse::<f64>)
        .collect::<Result<_, _>>()
        .map_err(|_| "Saisir trois nombres pour la position ou les dimensions")?;
    if values.len() != 3
        || values
            .iter()
            .any(|v| !v.is_finite() || (positive && *v <= 0.0))
    {
        return Err(
            "Saisir trois nombres valides ; les demi-dimensions doivent être positives".into(),
        );
    }
    Ok(())
}
fn set(document: &mut Document, key: &str, value: &str) -> Result<(), String> {
    if let Some(index) = document
        .properties()
        .iter()
        .position(|p| p.section.is_none() && p.key == key)
    {
        document.set_value(index, value).map_err(|e| e.to_string())
    } else {
        document
            .add_property(None, key, value)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
fn reference(path: &Path, pack_name: &str) -> Result<String, String> {
    let stem = path
        .file_stem()
        .and_then(|n| n.to_str())
        .ok_or("Nom de configuration invalide")?;
    Ok(format!("{pack_name}.{stem}"))
}
impl Creation {
    pub(super) fn build_guided(
        &self,
        catalog: &ParameterCatalog,
        pack: Option<&Pack>,
        documents: &BTreeMap<PathBuf, Document>,
    ) -> Result<Created, String> {
        if self.kind == Kind::Pack {
            return self.build();
        }
        let path = self.path()?;
        if self.kind.has_model() {
            named_path(
                self.kind.prefix(),
                self.name.trim().trim_end_matches(".dynx"),
            )?;
        }
        let mut document = Document::parse("").map_err(|e| e.to_string())?;
        for (key, value) in &self.fields {
            set(&mut document, key, value)?;
        }
        let new_pack = (self.guide.new_pack || pack.is_none()) && self.kind.has_model();
        let info = if new_pack {
            Some((self.pack_document()?, self.pack_name.trim().to_owned()))
        } else {
            None
        };
        let pack_name = if let Some((_, name)) = &info {
            name.clone()
        } else if let Some(p) = pack {
            let d = documents
                .get(Path::new("pack_info.dynx"))
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| p.read_config(0))?;
            cpm_core::PackMetadata::from_document(&d).name
        } else {
            self.pack_name.clone()
        };
        if self.kind.has_model()
            && (pack_name.trim().is_empty()
                || pack_name.chars().any(char::is_whitespace)
                || pack_name.contains(['.', ':', '/', '\\']))
        {
            return Err("Le nom interne du pack doit être un identifiant sans espace ni point pour les références".into());
        }
        let mut assets = BTreeMap::new();
        let mut configurations = Vec::new();
        let guide = &self.guide;
        if self.kind.has_model() {
            let model = guide.model.prepare(&folder(&path), &mut assets)?;
            set(&mut document, "Model", &model.to_string_lossy())?;
        }
        let mut extra = String::new();
        if self.kind.has_wheels() {
            let wheel = match guide.wheel_mode {
                WheelMode::Later => None,
                WheelMode::Existing => Some(
                    guide
                        .wheel_existing
                        .clone()
                        .ok_or("Choisir la roue à réutiliser dans le pack")?,
                ),
                WheelMode::New | WheelMode::Import => {
                    let wheel_path = named_path("wheel", &guide.wheel_name)?;
                    let mut wheel = if guide.wheel_mode == WheelMode::Import {
                        let p = guide
                            .wheel_import
                            .as_ref()
                            .ok_or("Choisir la configuration de roue à importer")?;
                        let bytes = read_config(p)?;
                        Document::parse(&bytes).map_err(|e| e.to_string())?
                    } else {
                        let mut d = Document::parse("").unwrap();
                        for (key, value) in &guide.wheel_fields {
                            set(&mut d, key, value)?;
                        }
                        d
                    };
                    if guide.wheel_mode == WheelMode::New
                        || guide.wheel_model.path(&folder(&wheel_path)).is_some()
                    {
                        let model = guide
                            .wheel_model
                            .prepare(&folder(&wheel_path), &mut assets)?;
                        set(&mut wheel, "Model", &model.to_string_lossy())?;
                    }
                    configurations.push((wheel_path.clone(), wheel));
                    Some(wheel_path)
                }
            };
            if let Some(wheel) = wheel {
                if guide.rows.is_empty() {
                    return Err(
                        "Ajouter au moins un emplacement de roue, ou choisir Plus tard".into(),
                    );
                }
                let mut names = BTreeSet::new();
                for row in &guide.rows {
                    if !row.name.starts_with("Wheel")
                        || !row
                            .name
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_')
                        || !names.insert(&row.name)
                    {
                        return Err("Chaque section de roue doit avoir un nom unique commençant par Wheel, sans espace".into());
                    }
                    check_vector(&row.position, false)
                        .map_err(|e| format!("{} : {e}", row.name))?;
                    let turn = if row.steering {
                        row.max_turn
                            .parse::<f64>()
                            .map_err(|_| "Angle de braquage invalide")?
                    } else {
                        0.0
                    };
                    if !turn.is_finite() || turn < 0.0 {
                        return Err("L’angle de braquage doit être un nombre positif".into());
                    }
                    let wheel_ref =
                        reference(row.existing.as_deref().unwrap_or(&wheel), &pack_name)?;
                    extra.push_str(&format!("\n{}{{\nAttachedWheel: {wheel_ref}\nPosition: {}\nDrivingWheel: {}\nIsRight: {}\nIsSteerable: {}\nMaxTurn: {turn}\nHandBrakingWheel: {}\n}}\n",row.name,row.position,row.driving,row.right,row.steering,row.handbrake));
                }
            }
            match guide.engine_mode {
                EngineMode::Later => {}
                EngineMode::Existing => set(
                    &mut document,
                    "DefaultEngine",
                    &reference(
                        guide
                            .engine_existing
                            .as_deref()
                            .ok_or("Choisir le moteur à réutiliser")?,
                        &pack_name,
                    )?,
                )?,
                EngineMode::New => {
                    let engine_path = named_path("engine", &guide.engine_name)?;
                    let mut engine = Document::parse("").unwrap();
                    for (key, value) in &guide.engine_fields {
                        set(&mut engine, key, value)?;
                    }
                    engine = engine_template(engine)?;
                    set(
                        &mut document,
                        "DefaultEngine",
                        &reference(&engine_path, &pack_name)?,
                    )?;
                    configurations.push((engine_path, engine));
                }
            }
            if let Some(path) = &guide.sounds {
                set(
                    &mut document,
                    "DefaultSounds",
                    &reference(path, &pack_name)?,
                )?;
            }
        }
        if self.kind.is_vehicle() && guide.seat {
            check_vector(&guide.seat_position, false)?;
            extra.push_str(&format!(
                "\nDriverSeat{{\nDriver: true\nPosition: {}\n}}\n",
                guide.seat_position
            ));
        }
        if self.kind.is_vehicle() && guide.steering {
            if guide.steering_object.trim().is_empty()
                || guide.steering_object.contains(['\r', '\n', '{', '}'])
            {
                return Err("Renseigner le nom de la pièce du volant dans l’OBJ".into());
            }
            extra.push_str(&format!(
                "\nSteeringWheel{{\nObjectName: {}\n",
                guide.steering_object
            ));
            if !guide.steering_position.trim().is_empty() {
                check_vector(&guide.steering_position, false)?;
                extra.push_str(&format!("Position: {}\n", guide.steering_position));
            }
            extra.push_str("}\n");
        }
        if self.kind.has_collisions() && guide.collision {
            check_vector(&guide.collision_position, false)?;
            check_vector(&guide.collision_scale, true)?;
            if self.kind.is_vehicle() {
                set(&mut document, "UseComplexCollisions", "false")?;
            } else {
                set(&mut document, "UseComplexCollision", "false")?;
            }
            extra.push_str(&format!(
                "\nShape_body{{\nType: BOX\nPosition: {}\nScale: {}\n}}\n",
                guide.collision_position, guide.collision_scale
            ));
        }
        document =
            Document::parse(&format!("{}{extra}", document.source())).map_err(|e| e.to_string())?;
        if self.kind == Kind::Engine {
            document = engine_template(document)?;
        }
        configurations.insert(0, (path.clone(), document));
        for (p, d) in &configurations {
            if !self.guide.allow_incomplete
                && p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("wheel_"))
            {
                let number = |key: &str| {
                    d.properties()
                        .iter()
                        .find(|p| p.section.is_none() && p.key == key)
                        .and_then(|p| p.value.parse::<f64>().ok())
                };
                if let (Some(radius), Some(rim), Some(width)) =
                    (number("Radius"), number("RimRadius"), number("Width"))
                {
                    if radius <= 0.0 || width <= 0.0 || rim < 0.0 || rim > radius {
                        return Err(format!("{} : Radius et Width doivent être positifs, et RimRadius doit être compris entre 0 et Radius",p.display()));
                    }
                }
            }
            let errors = catalog.errors(p, d);
            if !self.guide.allow_incomplete && !errors.is_empty() {
                return Err(format!("{} :\n{}", p.display(), errors.join("\n")));
            }
        }
        if !self.kind.has_model() && pack.is_none() {
            let (_, document) = configurations.remove(0);
            return Ok(Created::Configuration { path, document });
        }
        let temp;
        let mut candidate = if let Some((info, _)) = &info {
            temp = tempfile::tempdir().map_err(|e| e.to_string())?;
            std::fs::write(temp.path().join("pack_info.dynx"), info.source())
                .map_err(|e| e.to_string())?;
            Pack::open(temp.path())?
        } else {
            pack.ok_or("Aucun pack ouvert")?.clone()
        };
        candidate.add_bundle(configurations.clone(), assets.clone())?;
        let new_paths: BTreeSet<_> = configurations.iter().map(|(p, _)| p.clone()).collect();
        let audit_documents = if new_pack {
            BTreeMap::new()
        } else {
            documents.clone()
        };
        let errors: Vec<_> = candidate
            .audit(&audit_documents, catalog)
            .into_iter()
            .filter(|i| i.severity == cpm_core::Severity::Error && new_paths.contains(&i.path))
            .map(|i| format!("{} / {} : {}", i.path.display(), i.parameter, i.message))
            .collect();
        if !self.guide.allow_incomplete && !errors.is_empty() {
            return Err(errors.join("\n"));
        }
        Ok(Created::Bundle {
            primary: path,
            configurations,
            assets,
            new_pack: info,
        })
    }
    pub(super) fn pack_document(&self) -> Result<Document, String> {
        if self.pack_name.trim().is_empty() || self.file_version.trim().is_empty() {
            return Err("Renseigner le nom du pack et DcFileVersion".into());
        }
        let mut d = Document::parse("").unwrap();
        for (k, v) in [
            ("PackName", &self.pack_name),
            ("PackVersion", &self.version),
            ("Description", &self.description),
            ("DcFileVersion", &self.file_version),
        ] {
            set(&mut d, k, v)?;
        }
        Ok(d)
    }
}
fn engine_template(document: Document) -> Result<Document, String> {
    let rpm = document
        .properties()
        .iter()
        .find(|p| p.section.is_none() && p.key == "MaxRPM")
        .and_then(|p| p.value.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 1.0 && *v <= i32::MAX as f64)
        .ok_or("Pour créer la courbe du moteur, MaxRPM doit être compris entre 1 et 2147483647")?;
    let high = rpm.floor() as i32;
    let low = high.min(800);
    let mut source=format!("{}\n// Courbe de départ à adapter au moteur\nPoint_0{{\nRPMPower: 0 0\n}}\nPoint_1{{\nRPMPower: {} 0.75\n}}\nPoint_2{{\nRPMPower: {rpm} 1\n}}\n",document.source(),rpm*0.6);
    // DynamX assigns gear IDs in declaration order: reverse, neutral, then forward gears.
    for (index, (start, end)) in [(0, -30), (0, 0), (0, 50), (35, 100), (80, 160)]
        .into_iter()
        .enumerate()
    {
        source.push_str(&format!(
            "\nGear_{index}{{\nSpeedRange: {start} {end}\nRPMRange: {low} {high}\n}}\n"
        ));
    }
    Document::parse(&source).map_err(|e| e.to_string())
}
fn read_config(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("Configuration trop volumineuse".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn triangle(path: &Path, name: &str) {
        std::fs::write(
            path,
            format!("o {name}\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n"),
        )
        .unwrap();
    }
    fn root() -> (tempfile::TempDir, Pack) {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("pack_info.dynx"),
            "PackName: Demo\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        triangle(&root.path().join("car.obj"), "body");
        triangle(&root.path().join("wheel.obj"), "tire");
        let pack = Pack::open(root.path()).unwrap();
        (root, pack)
    }
    fn wheel(catalog: &ParameterCatalog) -> Document {
        let mut fields = Vec::new();
        let mut loaded = None;
        refresh_fields(
            &mut fields,
            &mut loaded,
            catalog,
            Path::new("wheel_existing.dynx"),
        );
        let mut d = Document::parse("// existing wheel\n").unwrap();
        for (key, value) in fields {
            set(
                &mut d,
                &key,
                if key == "Model" { "wheel.obj" } else { &value },
            )
            .unwrap();
        }
        d
    }
    fn form(catalog: &ParameterCatalog) -> Creation {
        let mut form = Creation::new(true);
        form.name = "car".into();
        let path = form.path().unwrap();
        form.refresh(catalog, &path);
        form.guide.model.existing = Some(PathBuf::from("car.obj"));
        form
    }
    #[test]
    fn reuses_existing_wheels_and_unsaved_pack_name_without_duplicate_files() {
        let catalog = ParameterCatalog::builtin();
        let (_root, mut pack) = root();
        pack.add_configuration(Path::new("wheel_existing.dynx"), wheel(&catalog))
            .unwrap();
        let mut form = form(&catalog);
        form.guide.wheel_mode = WheelMode::Existing;
        form.guide.wheel_existing = Some(PathBuf::from("wheel_existing.dynx"));
        let mut docs = BTreeMap::new();
        docs.insert(
            PathBuf::from("pack_info.dynx"),
            Document::parse("PackName: Renamed\nDcFileVersion: 12.5.0\n").unwrap(),
        );
        let Created::Bundle {
            configurations,
            assets,
            new_pack,
            ..
        } = form.build_guided(&catalog, Some(&pack), &docs).unwrap()
        else {
            panic!()
        };
        assert!(new_pack.is_none());
        assert!(assets.is_empty());
        assert_eq!(configurations.len(), 1);
        let d = &configurations[0].1;
        assert_eq!(
            d.sections()
                .iter()
                .filter(|s| s.name.starts_with("Wheel"))
                .count(),
            4
        );
        assert!(d.source().contains("AttachedWheel: Renamed.wheel_existing"));
        assert!(catalog.errors(&configurations[0].0, d).is_empty());
        assert_eq!(pack.configs().len(), 2);
    }
    #[test]
    fn creates_a_new_pack_with_models_wheels_engine_collisions_then_previews_and_exports() {
        let catalog = ParameterCatalog::builtin();
        let source = tempfile::tempdir().unwrap();
        triangle(&source.path().join("car.obj"), "body");
        triangle(&source.path().join("wheel.obj"), "tire");
        let mut form = form(&catalog);
        form.guide.model = ModelChoice {
            existing: None,
            files: ModelFiles {
                obj: Some(source.path().join("car.obj")),
                untextured: true,
                ..Default::default()
            },
        };
        form.guide.wheel_mode = WheelMode::New;
        form.guide.wheel_model.files = ModelFiles {
            obj: Some(source.path().join("wheel.obj")),
            untextured: true,
            ..Default::default()
        };
        let wheel_path = named_path("wheel", &form.guide.wheel_name).unwrap();
        refresh_fields(
            &mut form.guide.wheel_fields,
            &mut form.guide.wheel_path,
            &catalog,
            &wheel_path,
        );
        form.guide.engine_mode = EngineMode::New;
        let engine_path = named_path("engine", &form.guide.engine_name).unwrap();
        refresh_fields(
            &mut form.guide.engine_fields,
            &mut form.guide.engine_path,
            &catalog,
            &engine_path,
        );
        form.guide.collision = true;
        let created = form.build_guided(&catalog, None, &BTreeMap::new()).unwrap();
        let mut app = crate::CpmApp::default();
        app.create_document(created).unwrap();
        let pack = app.pack.as_ref().unwrap();
        assert_eq!(pack.configs().len(), 4);
        assert_eq!(pack.staged_resource_count(), 2);
        assert!(app.draft_pack.is_some());
        assert!(app.dirty());
        let editor = app.editor.as_ref().unwrap();
        let mesh = cpm_core::ObjRequest::resolve(Some(pack), &editor.path, &editor.document)
            .unwrap()
            .with_documents(app.documents())
            .load()
            .unwrap();
        assert_eq!(mesh.triangles, 5);
        assert_eq!(mesh.collisions.len(), 1);
        assert!(mesh.warnings.is_empty(), "{:?}", mesh.warnings);
        let destination = tempfile::tempdir().unwrap();
        let info = pack.read_config(0).unwrap();
        let target = pack
            .export_with_catalog(
                &cpm_core::ExportOptions {
                    destination: destination.path().into(),
                    output_name: "created".into(),
                    format: cpm_core::ExportFormat::Zip,
                    replace_existing: false,
                    metadata: cpm_core::PackMetadata::from_document(&info),
                },
                &app.documents(),
                &catalog,
            )
            .unwrap();
        let exported = Pack::open(&target).unwrap();
        assert!(exported
            .audit(&BTreeMap::new(), &catalog)
            .iter()
            .all(|i| i.severity != cpm_core::Severity::Error));
        assert_eq!(exported.obj_resources().len(), 2);
        assert!(!source.path().join("vehicle_car.dynx").exists());
    }
    #[test]
    fn imports_wheel_configuration_relinks_its_model_and_keeps_the_original() {
        let catalog = ParameterCatalog::builtin();
        let (_root, pack) = root();
        let external = tempfile::tempdir().unwrap();
        triangle(&external.path().join("wheel.obj"), "tire");
        let original = wheel(&catalog).source().to_owned();
        std::fs::write(external.path().join("wheel_old.dynx"), &original).unwrap();
        let mut form = form(&catalog);
        form.guide.wheel_mode = WheelMode::Import;
        form.guide.wheel_name = "imported".into();
        form.guide.wheel_import = Some(external.path().join("wheel_old.dynx"));
        form.guide.wheel_model.files.obj = Some(external.path().join("wheel.obj"));
        let Created::Bundle {
            configurations,
            assets,
            ..
        } = form
            .build_guided(&catalog, Some(&pack), &BTreeMap::new())
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(configurations.len(), 2);
        let imported = &configurations[1].1;
        assert!(imported.source().starts_with("// existing wheel"));
        assert!(imported
            .source()
            .contains("models/wheel_imported/model.obj"));
        assert_eq!(assets.len(), 1);
        assert_eq!(
            std::fs::read_to_string(external.path().join("wheel_old.dynx")).unwrap(),
            original
        );
    }
    #[test]
    fn invalid_positions_missing_files_and_duplicate_names_do_not_modify_pack() {
        let catalog = ParameterCatalog::builtin();
        let (_root, mut pack) = root();
        pack.add_configuration(Path::new("wheel_existing.dynx"), wheel(&catalog))
            .unwrap();
        let before = pack.configs().to_vec();
        let mut form = form(&catalog);
        form.guide.wheel_mode = WheelMode::Existing;
        form.guide.wheel_existing = Some(PathBuf::from("wheel_existing.dynx"));
        form.guide.rows[0].position = "bad".into();
        assert!(form
            .build_guided(&catalog, Some(&pack), &BTreeMap::new())
            .is_err());
        form.guide.rows[0].position = "0 0 0".into();
        form.guide.model = ModelChoice::default();
        assert!(form
            .build_guided(&catalog, Some(&pack), &BTreeMap::new())
            .is_err());
        assert_eq!(pack.configs(), before);
        form.guide.model.existing = Some(PathBuf::from("car.obj"));
        pack.add_configuration(
            Path::new("vehicle_car.dynx"),
            Document::parse("Name: existing\n").unwrap(),
        )
        .unwrap();
        assert!(form
            .build_guided(&catalog, Some(&pack), &BTreeMap::new())
            .is_err());
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::*;
    #[test]
    fn failed_bundle_keeps_current_editor_and_resources_unchanged() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("pack_info.dynx"),
            "PackName: Demo\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        let mut app = crate::CpmApp::default();
        app.open_selection(root.path());
        app.editor
            .as_mut()
            .unwrap()
            .document
            .set_value(0, "Unsaved")
            .unwrap();
        let mut assets = BTreeMap::new();
        assets.insert(PathBuf::from("models/new.obj"), b"data".to_vec());
        let result = app.create_document(Created::Bundle {
            primary: PathBuf::from("pack_info.dynx"),
            configurations: vec![(
                PathBuf::from("pack_info.dynx"),
                Document::parse("PackName: overwrite\n").unwrap(),
            )],
            assets,
            new_pack: None,
        });
        assert!(result.is_err());
        assert_eq!(
            app.editor.as_ref().unwrap().document.properties()[0].value,
            "Unsaved"
        );
        assert_eq!(app.pack.as_ref().unwrap().staged_resource_count(), 0);
        assert!(!root.path().join("models").exists());
    }
    #[test]
    fn incomplete_addon_parameters_require_an_explicit_draft_choice() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("pack_info.dynx"),
            "PackName: Demo\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join("car.obj"),
            "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
        )
        .unwrap();
        let pack = Pack::open(root.path()).unwrap();
        let mut catalog = ParameterCatalog::builtin();
        catalog.files[0]
            .catalog
            .rules
            .iter_mut()
            .find(|r| r.file_pattern == "vehicle_*.dynx" && r.section_pattern.is_none())
            .unwrap()
            .parameters
            .push(cpm_core::ParameterDefinition {
                name: "AddonRequired".into(),
                allow_empty: false,
                required: true,
                ..Default::default()
            });
        let mut form = Creation::new(true);
        let path = form.path().unwrap();
        form.refresh(&catalog, &path);
        form.guide.model.existing = Some(PathBuf::from("car.obj"));
        assert!(form
            .build_guided(&catalog, Some(&pack), &BTreeMap::new())
            .is_err());
        form.guide.allow_incomplete = true;
        assert!(matches!(
            form.build_guided(&catalog, Some(&pack), &BTreeMap::new()),
            Ok(Created::Bundle { .. })
        ));
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;
    #[test]
    fn starter_engine_has_curve_and_ordered_reverse_neutral_and_forward_gears() {
        let catalog = ParameterCatalog::builtin();
        let mut form = Creation::new(false);
        form.kind = Kind::Engine;
        let path = form.path().unwrap();
        form.refresh(&catalog, &path);
        let Created::Configuration { path, document } =
            form.build_guided(&catalog, None, &BTreeMap::new()).unwrap()
        else {
            panic!()
        };
        assert!(catalog.errors(&path, &document).is_empty());
        assert_eq!(
            document
                .sections()
                .iter()
                .filter(|s| s.name.starts_with("Gear_"))
                .count(),
            5
        );
        assert!(
            document.source().find("SpeedRange: 0 -30").unwrap()
                < document.source().find("SpeedRange: 0 0").unwrap()
        );
        assert!(document.source().contains("RPMPower: 6000 1"));
    }
}

#[cfg(test)]
mod gltf_gate_tests {
    use super::*;
    #[test]
    fn reusing_a_gltf_requires_beta_even_when_skipping_property_errors() {
        let mut choice = ModelChoice {
            existing: Some("models/car.gltf".into()),
            files: ModelFiles::default(),
        };
        let folder = Path::new("models/new");
        assert!(choice.path(folder).is_none());
        assert!(choice.prepare(folder, &mut BTreeMap::new()).is_err());
        choice.files.dynamx_beta_confirmed = true;
        assert_eq!(choice.path(folder), choice.existing);
        assert_eq!(
            choice.prepare(folder, &mut BTreeMap::new()).unwrap(),
            PathBuf::from("models/car.gltf")
        );
        choice.files.dynamx_beta_confirmed = false;
        assert!(choice.prepare(folder, &mut BTreeMap::new()).is_err());
        choice.existing = Some("models/car.obj".into());
        assert!(choice.path(folder).is_some());
    }
}
