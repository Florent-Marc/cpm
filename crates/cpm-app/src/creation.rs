mod guided;
use cpm_core::{Document, Pack, ParameterCatalog};
use eframe::egui;
use guided::{Guide, Step};
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, sync::mpsc};

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
    fn has_model(self) -> bool {
        matches!(
            self,
            Self::Vehicle
                | Self::Wheel
                | Self::Trailer
                | Self::Boat
                | Self::Helicopter
                | Self::Block
                | Self::Prop
                | Self::Item
                | Self::Armor
        )
    }
    fn is_vehicle(self) -> bool {
        matches!(
            self,
            Self::Vehicle | Self::Trailer | Self::Boat | Self::Helicopter
        )
    }
    fn has_wheels(self) -> bool {
        matches!(self, Self::Vehicle | Self::Trailer)
    }
    fn has_collisions(self) -> bool {
        self.is_vehicle() || matches!(self, Self::Block | Self::Prop)
    }
    fn has_assembly(self) -> bool {
        self.has_collisions()
    }
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
    Bundle {
        primary: PathBuf,
        configurations: Vec<(PathBuf, Document)>,
        assets: BTreeMap<PathBuf, Vec<u8>>,
        new_pack: Option<(Document, String)>,
    },
    Pack {
        document: Document,
        name: String,
    },
    Configuration {
        path: PathBuf,
        document: Document,
    },
}
pub enum Action {
    None,
    Close,
    Create(Created),
}
pub struct Creation {
    guide: Guide,
    worker: Option<mpsc::Receiver<Result<Created, String>>>,
    prepared: Option<Created>,
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
            guide: Guide::default(),
            worker: None,
            prepared: None,
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
                        p.default_value
                            .or_else(|| guided::initial(&p.name).map(str::to_owned))
                            .unwrap_or_default()
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
    fn snapshot(&self) -> Self {
        Self {
            guide: self.guide.clone(),
            worker: None,
            prepared: None,
            kind: self.kind,
            name: self.name.clone(),
            folder: self.folder.clone(),
            pack_name: self.pack_name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            file_version: self.file_version.clone(),
            fields: self.fields.clone(),
            loaded_path: self.loaded_path.clone(),
            optional: self.optional,
            error: None,
        }
    }
    fn start_prepare(
        &mut self,
        ctx: &egui::Context,
        catalog: &ParameterCatalog,
        pack: Option<&Pack>,
        documents: BTreeMap<PathBuf, Document>,
    ) {
        let form = self.snapshot();
        let catalog = catalog.clone();
        let pack = pack.cloned();
        let ctx = ctx.clone();
        let (tx, rx) = mpsc::channel();
        self.worker = Some(rx);
        self.prepared = None;
        self.error = None;
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                form.build_guided(&catalog, pack.as_ref(), &documents)
            }))
            .unwrap_or_else(|_| {
                Err("La préparation a été interrompue ; vérifier les fichiers sélectionnés".into())
            });
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    fn step_error(
        &self,
        step: Step,
        catalog: &ParameterCatalog,
        pack: Option<&Pack>,
    ) -> Option<String> {
        match step {
            Step::Type=>{
                if self.kind==Kind::Pack {return self.pack_document().and_then(|d|{let e=catalog.errors(Path::new("pack_info.dynx"),&d);if e.is_empty(){Ok(())}else{Err(e.join("\n"))}}).err();}
                if let Err(error)=self.path(){return Some(error);}
                if self.kind.has_model(){if let Err(error)=guided::named_path(self.kind.prefix(),self.name.trim().trim_end_matches(".dynx")){return Some(error);}}
                if self.kind.has_model()&&(pack.is_none()||self.guide.new_pack){return self.pack_document().map(|_|()).err();}
                None
            }
            Step::Assets=>self.path().ok().and_then(|p|self.guide.model.path(&guided::folder(&p))).is_none().then(||"Choisir un modèle avant de continuer. Pour un glTF, confirmer l’utilisation de DynamX bêta. Sélectionner aussi ses fichiers BIN et images.".into()),
            Step::Properties=>{
                if self.guide.allow_incomplete{return None;}
                let path=self.path().ok()?;let mut document=Document::parse("").unwrap();
                for (k,v) in &self.fields{if let Err(e)=document.add_property(None,k,v){return Some(e.to_string());}}
                if self.kind.has_model(){if let Some(model)=self.guide.model.path(&guided::folder(&path)){if let Some(index)=document.properties().iter().position(|p|p.key=="Model"){let _=document.set_value(index,&model.to_string_lossy());}else{let _=document.add_property(None,"Model",&model.to_string_lossy());}}}
                let errors=catalog.errors(&path,&document);(!errors.is_empty()).then(||errors.join("\n"))
            }
            _=>None
        }
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        catalog: &ParameterCatalog,
        pack: Option<&Pack>,
        documents: BTreeMap<PathBuf, Document>,
        can_replace: bool,
    ) -> Action {
        if let Some(rx) = &self.worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.worker = None;
                    match result {
                        Ok(created) => self.prepared = Some(created),
                        Err(error) => self.error = Some(error),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.worker = None;
                    self.error = Some("Préparation interrompue".into());
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100))
                }
            }
        }
        let mut open = true;
        let mut action = Action::None;
        let mut next = false;
        let mut back = false;
        let mut prepare = false;
        let busy = self.worker.is_some();
        egui::Window::new("Créer — assistant guidé").open(&mut open).collapsible(false).default_size([760.0,650.0]).min_size([520.0,400.0]).show(ctx,|ui|{
            let steps=Guide::steps(self.kind);self.guide.step=self.guide.step.min(steps.len()-1);let current=steps[self.guide.step];
            ui.heading(format!("Étape {} sur {} — {}",self.guide.step+1,steps.len(),current.title()));
            ui.horizontal_wrapped(|ui|{for (i,step) in steps.iter().enumerate(){let text=format!("{} {}",i+1,step.title());ui.label(if i==self.guide.step{egui::RichText::new(text).strong()}else{egui::RichText::new(text).weak()});}});ui.separator();
            let target_pack=pack.filter(|_|!self.guide.new_pack);
            egui::ScrollArea::vertical().id_salt("creation-step").max_height((ui.available_height()-200.0).max(140.0)).show(ui,|ui|{
                ui.add_enabled_ui(!busy,|ui|match current {
                    Step::Type=>{
                        ui.label("Que souhaitez-vous ajouter ? Choisissez le type, puis donnez un nom court au fichier. Les étapes suivantes vous aideront à relier le modèle et ses paramètres.");
                        egui::ComboBox::from_id_salt("creation-kind").selected_text(self.kind.label()).show_ui(ui,|ui|{for kind in Kind::ALL {if ui.selectable_value(&mut self.kind,kind,kind.label()).changed(){self.fields.clear();self.loaded_path=None;self.guide=Guide::default();self.prepared=None;self.error=None;}}});
                        if self.kind!=Kind::Pack {
                            ui.horizontal(|ui|{ui.label("Nom du fichier, sans préfixe ni extension");ui.text_edit_singleline(&mut self.name);});
                            ui.weak("Exemple : transit → vehicle_transit.dynx. Utilisez des lettres, chiffres, tirets ou underscores.");
                            if pack.is_some(){ui.horizontal(|ui|{ui.label("Sous-dossier facultatif");ui.text_edit_singleline(&mut self.folder);});if self.kind.has_model(){ui.checkbox(&mut self.guide.new_pack,"Créer un nouveau pack au lieu d’ajouter au pack ouvert");}}
                            if let Ok(path)=self.path(){ui.monospace(path.display().to_string());}
                        }
                        if self.kind==Kind::Pack || (self.kind.has_model()&&(pack.is_none()||self.guide.new_pack)) {
                            ui.separator();ui.label("Un nouveau pack sera préparé. Vous choisirez où le sauvegarder avec File → Exporter.");
                            for (label,value) in [("Nom interne du pack",&mut self.pack_name),("Version du pack",&mut self.version),("Version des fichiers DynamX (DcFileVersion)",&mut self.file_version),("Description du pack",&mut self.description)]{ui.horizontal(|ui|{ui.label(label);ui.text_edit_singleline(value);});}
                            ui.weak("Le nom interne sert aux références entre configurations : choisissez un identifiant sans espace ni point. DcFileVersion doit correspondre à la version de DynamX visée.");
                        }else{ui.weak(if pack.is_some(){"Les fichiers seront ajoutés au pack ouvert lors de la création, puis sauvegardés à l’export."}else{"Cette configuration sera créée seule ; File → Exporter permettra de l’enregistrer."});}
                    }
                    Step::Assets=>{
                        ui.label("Choisir un OBJ avec ses MTL et textures, ou un glTF avec ses fichiers BIN et images. Le glTF nécessite DynamX bêta ; le GLB n’est pas accepté.");
                        self.guide.model.ui(ui,target_pack,"object-model");
                        if let Ok(path)=self.path(){if let Some(model)=self.guide.model.path(&guided::folder(&path)){ui.separator();ui.label("Chemin du modèle dans le pack après import");ui.monospace(model.display().to_string());}}
                        ui.weak("Vos originaux ne sont pas modifiés. Les dépendances seront vérifiées et leurs chemins réécrits automatiquement avant la création.");
                    }
                    Step::Properties=>{
                        ui.label("Ces propriétés décrivent votre objet. Les valeurs proposées sont un point de départ, pas un réglage définitif : adaptez notamment la masse et les dimensions. Passez la souris sur un nom pour lire son explication.");
                        if ui.checkbox(&mut self.optional,"Afficher aussi les paramètres facultatifs ayant une valeur par défaut").changed(){self.loaded_path=None;}
                        if let Ok(path)=self.path(){self.refresh(catalog,&path);guided::fields_ui(ui,&mut self.fields,catalog,&path,self.kind.has_model());}
                        ui.checkbox(&mut self.guide.allow_incomplete,"Continuer en brouillon si des paramètres restent à compléter");
                        if self.guide.allow_incomplete{ui.colored_label(egui::Color32::YELLOW,"Les erreurs restantes bloqueront l’export. Vous pourrez les corriger dans l’éditeur.");}
                    }
                    Step::Assembly=>self.guide.assembly_ui(ui,catalog,target_pack,self.kind),
                    Step::Summary=>{
                        ui.label("Vérifiez ce qui sera créé. Rien n’est encore ajouté au pack. Revenez aux étapes précédentes pour corriger un fichier, une position ou une propriété.");
                        if self.kind.has_wheels(){if self.guide.wheel_mode==guided::WheelMode::Later{ui.colored_label(egui::Color32::YELLOW,"Création sans roues : le véhicule restera à compléter.");}
                        if self.guide.engine_mode==guided::EngineMode::Later{ui.colored_label(egui::Color32::YELLOW,"Création sans moteur : le véhicule ne sera pas prêt à rouler.");}
                        if self.guide.engine_mode==guided::EngineMode::New{ui.weak("Le moteur et ses rapports sont des réglages de départ : adaptez-les au véhicule avant un test en jeu.");}}
                        ui.weak("Après création, ouvrez l’aperçu 3D pour vérifier les textures et ajuster les positions. Testez ensuite le comportement dans DynamX.");
                        if let Some(prepared)=&self.prepared {match prepared {
                            Created::Pack{name,..}=>{ui.label(format!("Nouveau pack : {name}"));}
                            Created::Configuration{path,..}=>{ui.monospace(path.display().to_string());}
                            Created::Bundle{primary,configurations,assets,new_pack}=>{
                                if let Some((_,name))=new_pack{ui.label(format!("Nouveau pack : {name}"));}ui.label(format!("Objet principal : {}",primary.display()));
                                ui.label(format!("{} configuration(s), {} ressource(s), {:.1} Mio",configurations.len(),assets.len(),assets.values().map(Vec::len).sum::<usize>() as f64/(1024.0*1024.0)));
                                ui.collapsing("Configurations",|ui|{for (path,document) in configurations{ui.collapsing(path.display().to_string(),|ui|{ui.monospace(document.source());});}});
                                ui.collapsing("Ressources importées",|ui|{for path in assets.keys(){ui.monospace(path.display().to_string());}});
                            }
                        }}
                        if !busy&&self.prepared.is_none()&&ui.button("Vérifier les fichiers et préparer la création").clicked(){prepare=true;}
                    }
                });
            });
            if busy{ui.horizontal(|ui|{ui.spinner();ui.label("Lecture des fichiers, vérification des dépendances et préparation…");});}
            let allowed=(self.kind!=Kind::Pack&&pack.is_some()&&!self.guide.new_pack)||can_replace;
            if !allowed{ui.colored_label(egui::Color32::YELLOW,"Enregistrez ou annulez les modifications avant de créer un autre pack.");}
            let error=self.step_error(current,catalog,pack);
            if error.is_some()||self.error.is_some(){egui::ScrollArea::vertical().id_salt("creation-errors").max_height(110.0).show(ui,|ui|{
                if let Some(message)=&error{ui.colored_label(egui::Color32::LIGHT_RED,message);}
                if let Some(message)=&self.error{ui.colored_label(egui::Color32::LIGHT_RED,message);}
            });}
            ui.separator();ui.horizontal(|ui|{
                if ui.add_enabled(!busy&&self.guide.step>0,egui::Button::new("← Précédent")).clicked(){back=true;}
                if current==Step::Summary {
                    if ui.add_enabled(!busy&&allowed&&self.prepared.is_some(),egui::Button::new("Créer dans l’éditeur")).clicked(){if let Some(prepared)=self.prepared.take(){action=Action::Create(prepared);}}
                }else if ui.add_enabled(!busy&&allowed&&error.is_none(),egui::Button::new("Suivant →")).clicked(){next=true;}
                if ui.button("Annuler").clicked(){action=Action::Close;}
            });
        });
        if back {
            self.guide.step -= 1;
            self.prepared = None;
            self.error = None;
        }
        if next {
            self.guide.step += 1;
            self.error = None;
            self.prepared = None;
            if Guide::steps(self.kind)[self.guide.step] == Step::Summary {
                prepare = true;
            }
        }
        if prepare {
            self.start_prepare(ctx, catalog, pack, documents);
        }
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

#[cfg(test)]
mod wizard_ui_tests {
    use super::*;
    fn label(shapes: &[egui::epaint::ClippedShape], name: &str) -> egui::Pos2 {
        fn find(shape: &egui::epaint::Shape, name: &str) -> Option<egui::Pos2> {
            match shape {
                egui::epaint::Shape::Text(t) if t.galley.job.text == name => {
                    Some(t.pos + t.galley.rect.center().to_vec2())
                }
                egui::epaint::Shape::Vec(v) => v.iter().find_map(|s| find(s, name)),
                _ => None,
            }
        }
        shapes
            .iter()
            .find_map(|s| find(&s.shape, name))
            .unwrap_or_else(|| panic!("missing control: {name}"))
    }
    fn frame(
        form: &mut Creation,
        ctx: &egui::Context,
        catalog: &ParameterCatalog,
        pack: &Pack,
        events: Vec<egui::Event>,
    ) -> (Vec<egui::epaint::ClippedShape>, Action) {
        let mut action = Action::None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1400.0, 1000.0),
                )),
                events,
                ..Default::default()
            },
            |_| {
                action = form.show(ctx, catalog, Some(pack), BTreeMap::new(), true);
            },
        );
        output.textures_delta.clear();
        (output.shapes, action)
    }
    fn click(
        form: &mut Creation,
        ctx: &egui::Context,
        catalog: &ParameterCatalog,
        pack: &Pack,
        name: &str,
    ) -> Action {
        frame(form, ctx, catalog, pack, vec![]);
        let (shapes, _) = frame(form, ctx, catalog, pack, vec![]);
        let pos = label(&shapes, name);
        frame(
            form,
            ctx,
            catalog,
            pack,
            vec![egui::Event::PointerMoved(pos)],
        );
        frame(
            form,
            ctx,
            catalog,
            pack,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(
            form,
            ctx,
            catalog,
            pack,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        )
        .1
    }
    #[test]
    fn wizard_moves_forward_backward_preserves_edits_and_requires_review_before_creation() {
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
        let catalog = ParameterCatalog::builtin();
        let ctx = egui::Context::default();
        let mut form = Creation::new(true);
        click(&mut form, &ctx, &catalog, &pack, "Suivant →");
        assert_eq!(form.guide.step, 1);
        click(&mut form, &ctx, &catalog, &pack, "Suivant →");
        assert_eq!(form.guide.step, 1); // No selected OBJ yet.
        form.guide.model.existing = Some(PathBuf::from("car.obj"));
        click(&mut form, &ctx, &catalog, &pack, "Suivant →");
        assert_eq!(form.guide.step, 2);
        frame(&mut form, &ctx, &catalog, &pack, vec![]);
        form.fields.iter_mut().find(|(k, _)| k == "Name").unwrap().1 = "My vehicle".into();
        click(&mut form, &ctx, &catalog, &pack, "← Précédent");
        assert_eq!(form.guide.step, 1);
        click(&mut form, &ctx, &catalog, &pack, "Suivant →");
        assert_eq!(
            form.fields.iter().find(|(k, _)| k == "Name").unwrap().1,
            "My vehicle"
        );
        click(&mut form, &ctx, &catalog, &pack, "Suivant →");
        assert_eq!(form.guide.step, 3);
        click(&mut form, &ctx, &catalog, &pack, "Suivant →");
        assert_eq!(form.guide.step, 4);
        for _ in 0..200 {
            frame(&mut form, &ctx, &catalog, &pack, vec![]);
            if form.prepared.is_some() || form.error.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(form.prepared.is_some(), "{:?}", form.error);
        assert_eq!(pack.configs().len(), 1); // Review does not add anything.
        assert!(matches!(
            click(&mut form, &ctx, &catalog, &pack, "Créer dans l’éditeur"),
            Action::Create(Created::Bundle { .. })
        ));
        assert!(!root.path().join("vehicle_nouveau.dynx").exists());
    }
}
