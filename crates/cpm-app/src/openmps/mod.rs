pub mod client;
mod servers;
use client::{Capabilities, Client, Profile, Publication, Version};
use cpm_core::{Document, Pack, PackMetadata, ParameterCatalog, PreparedOpenMps};
use eframe::egui;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc,
    },
};
#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum Tab {
    #[default]
    Packs,
    Publish,
    Servers,
}
enum Completed {
    Connected(Capabilities, Vec<Version>),
    Prepared(Arc<PreparedOpenMps>, u64),
    Published(Publication),
    Downloaded(PathBuf),
    Recovered(PathBuf),
}
#[derive(Default)]
pub struct OpenMps {
    pub open: bool,
    profiles: Vec<Profile>,
    selected: Option<String>,
    sessions: BTreeMap<String, String>,
    remember_token: bool,
    tab: Tab,
    remote_filter: String,
    confirm_delete: bool,
    confirm_publish: bool,
    profile_name: String,
    endpoint: String,
    token: String,
    access_key: String,
    caps: Option<Capabilities>,
    versions: Vec<Version>,
    key: String,
    version: String,
    prepared: Option<(Arc<PreparedOpenMps>, u64)>,
    publication: Option<Publication>,
    receiver: Option<mpsc::Receiver<Result<Completed, String>>>,
    progress: Arc<AtomicU64>,
    total: u64,
    status: String,
    error: Option<String>,
    recovery_parent: String,
    recovered: Option<PathBuf>,
    open_request: Option<PathBuf>,
}
impl OpenMps {
    fn profiles_path() -> Result<PathBuf, String> {
        Ok(std::env::current_exe()
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("Dossier du programme introuvable")?
            .join("openmps-servers.json"))
    }
    pub fn load_profiles(&mut self) {
        match Self::profiles_path().and_then(|p| client::load_profiles(&p)) {
            Ok(profiles) => {
                self.profiles = profiles;
                if let Some(id) = self
                    .profiles
                    .iter()
                    .find(|p| p.active)
                    .or_else(|| self.profiles.first())
                    .map(|p| p.id.clone())
                {
                    self.select_server(&id, &servers::SystemVault);
                }
            }
            Err(e) => self.error = Some(format!("Profils OpenMPS : {e}")),
        }
    }
    pub fn begin(&mut self, metadata: Option<PackMetadata>) {
        if !self.open {
            if let Some(info) = metadata {
                self.key = info
                    .name
                    .chars()
                    .map(|c| {
                        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                            c.to_ascii_lowercase()
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>()
                    .trim_matches('_')
                    .chars()
                    .take(64)
                    .collect();
                if self.key.is_empty() {
                    self.key = "monpack".into();
                }
                self.version = if cpm_core::validate_mps_version(&info.version).is_ok() {
                    info.version
                } else {
                    "1.0.0".into()
                };
            }
            if self.access_key.is_empty() {
                self.access_key = "legacy".into();
            }
        }
        self.open = true;
    }
    pub fn take_open_request(&mut self) -> Option<PathBuf> {
        self.open_request.take()
    }
    pub fn busy(&self) -> bool {
        self.receiver.is_some()
    }
    fn start(
        &mut self,
        ctx: &egui::Context,
        status: &str,
        job: impl FnOnce() -> Result<Completed, String> + Send + 'static,
    ) {
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        self.error = None;
        self.status = status.into();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)).unwrap_or_else(|_|Err("Opération OpenMPS interrompue ; vérifier les versions distantes avant de relancer une publication".into()));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    fn client(&self) -> Result<Client, String> {
        Client::new(&self.endpoint, &self.token)
    }
    fn poll(&mut self) {
        let result = match self.receiver.as_ref().map(mpsc::Receiver::try_recv) {
            Some(Ok(result)) => result,
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                Err("Traitement OpenMPS interrompu".into())
            }
            _ => return,
        };
        self.receiver = None;
        match result {
            Err(error) => self.error = Some(error),
            Ok(Completed::Connected(caps, versions)) => {
                self.caps = Some(caps);
                self.versions = versions;
                self.status = "Connexion validée ; versions actualisées".into();
            }
            Ok(Completed::Prepared(package, revision)) => {
                self.prepared = Some((package, revision));
                self.status =
                    "Copie de publication préparée ; vérifier le récapitulatif puis publier".into();
            }
            Ok(Completed::Published(publication)) => {
                let row = Version {
                    pack_key: publication.pack_key.clone(),
                    pack_version: publication.pack_version.clone(),
                    client_size: publication.client_size,
                    client_sha256: Some(publication.client_sha256.clone()),
                    files_count: publication.files_count,
                };
                self.versions.retain(|r| {
                    !(r.pack_key == row.pack_key && r.pack_version == row.pack_version)
                });
                self.versions.push(row);
                self.status = if publication.idempotent {
                    "Cette archive avait déjà été publiée ; aucune version n’a été remplacée".into()
                } else {
                    "Publication terminée ; le ZIP joueurs peut être téléchargé".into()
                };
                self.publication = Some(publication);
            }
            Ok(Completed::Recovered(path)) => {
                self.status = format!("Copie éditable récupérée : {}", path.display());
                self.recovered = Some(path);
            }
            Ok(Completed::Downloaded(path)) => {
                self.status = format!("ZIP joueurs enregistré : {}", path.display())
            }
        }
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        pack: Option<&Pack>,
        documents: BTreeMap<PathBuf, Document>,
        catalog: Arc<ParameterCatalog>,
        revision: u64,
        can_open: bool,
    ) {
        self.poll();
        if !self.open {
            return;
        }
        if self.prepared.as_ref().is_some_and(|(_, r)| *r != revision) {
            self.prepared = None;
            self.confirm_publish = false;
            self.status = "Le pack a changé ; préparer à nouveau la copie avant publication".into();
        }
        let busy = self.busy();
        let mut open = true;
        let mut connect = false;
        let mut prepare = false;
        let mut publish = false;
        let mut download = None;
        let mut recover = None;
        egui::Window::new("OpenMPS — publier et récupérer des packs").open(&mut open).default_size([780.0,680.0]).min_size([560.0,440.0]).collapsible(false).show(ctx,|ui|{
            ui.label("Cette fenêtre est facultative. Fichier → Exporter fonctionne toujours sans serveur ni connexion.");
            ui.add_enabled_ui(!busy, |ui| {
                    let mut selected=None;
                    let selected_name=self.profiles.iter().find(|p|Some(&p.id)==self.selected.as_ref()).map(|p|p.name.as_str()).unwrap_or("Choisir un serveur…");
                    ui.horizontal_wrapped(|ui|{
                        egui::ComboBox::from_id_salt("openmps-profiles").selected_text(selected_name).show_ui(ui,|ui|{for profile in &self.profiles{if ui.selectable_label(Some(&profile.id)==self.selected.as_ref(),&profile.name).clicked(){selected=Some(profile.id.clone());}}});
                        if ui.button("Gérer les serveurs…").clicked(){self.tab=Tab::Servers;}
                        if ui.button("Actualiser / connecter").clicked(){connect=true;}
                    });
                    if let Some(id)=selected{self.select_server(&id,&servers::SystemVault);self.persist_selection();}
                    ui.strong(self.destination());
                    ui.label(if self.busy(){"Connexion en cours"}else if self.caps.is_some(){"Connecté"}else{"Non connecté"});
                    if self.token.is_empty(){ui.weak("Ouvrir Gérer les serveurs pour ajouter une connexion ou saisir son jeton.");}
                    ui.collapsing("Configurer OpenMPS une première fois",|ui|{ui.label("Installer l’API d’administration v1 sur votre serveur, définir SERVER_PUBLIC_URL pour les joueurs, puis créer un jeton en ligne de commande :");ui.monospace("php tools/api_token.php create CPM");ui.label("Coller ici le jeton affiché. Ne pas utiliser le mot de passe du tableau de bord ni la clé MPS du jeu.");});
                    if let Some(caps)=&self.caps {
                        if let Some(scopes)=&caps.scopes { ui.label(format!("Droits : {}",scopes.join(", "))); }
                        else { ui.weak("Droits non détaillés par cette version de l’API."); }
                        ui.collapsing("Informations du serveur", |ui| {
                            ui.label(format!("API v{} · MPS {} · limite {:.0} Mio",caps.api_version,caps.mps_version,caps.max_upload_bytes as f64/1048576.0));
                            ui.label(format!("URL joueurs : {}",caps.public_url));
                        });
                    }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.tab, Tab::Packs, "Packs distants");
                    ui.selectable_value(&mut self.tab, Tab::Publish, "Publier");
                    ui.selectable_value(&mut self.tab, Tab::Servers, "Serveurs");
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().id_salt(("openmps-tab",self.tab as u8)).max_height((ui.available_height()-95.0).max(200.0)).show(ui,|ui|{
                ui.add_enabled_ui(!busy,|ui|{
                    if self.tab == Tab::Servers { self.server_controls(ui, ctx); }
                    if self.tab == Tab::Publish {
                    ui.heading("Préparer la publication");
                    ui.label("Le diagnostic contrôle le pack et prépare une copie avec les ressources sous assets/. Le pack de travail et ses exports classiques ne sont pas modifiés.");
                    ui.horizontal(|ui|{ui.label("Identifiant OpenMPS / PackName publié");if ui.text_edit_singleline(&mut self.key).changed(){self.prepared=None;self.publication=None;self.confirm_publish=false;}});
                    ui.horizontal(|ui|{ui.label("Version à publier");if ui.text_edit_singleline(&mut self.version).changed(){self.prepared=None;self.publication=None;self.confirm_publish=false;}});
                    ui.weak("L’identifiant et les références internes sont adaptés dans la copie. Une version déjà présente sur le serveur ne sera pas écrasée.");
                    if ui.add_enabled(pack.is_some(),crate::theme::primary("Vérifier et préparer la publication")).clicked(){prepare=true;}
                    if pack.is_none(){ui.weak("Ouvrir un pack pour publier. La liste distante reste consultable sans pack local.");}
                    if let Some((package,_))=&self.prepared {
                        let report=&package.report;let size=std::fs::metadata(&package.archive).map(|m|m.len()).unwrap_or(0);
                        ui.label(format!("{} {} · {} fichiers · {:.1} Mio",package.key,package.version,report.files,size as f64/1048576.0));
                        ui.collapsing(format!("Ressources déplacées dans la copie ({})",report.moved.len()),|ui|{for (from,to) in &report.moved{ui.label(format!("{} → {}",from.display(),to.display()));}});
                        if !report.skipped.is_empty(){ui.collapsing("Fichiers de documentation ou anciens fichiers MPS exclus",|ui|{for p in &report.skipped{ui.label(p.display().to_string());}});}
                        for warning in &report.warnings{ui.colored_label(crate::theme::WARNING,warning);}
                        if self.caps.as_ref().is_some_and(|caps|size>caps.max_upload_bytes){ui.colored_label(crate::theme::ERROR,"L’archive dépasse la limite annoncée par le serveur.");}
                    }
                    ui.add_space(18.0);ui.heading("Publier la version");
                    ui.strong(format!("Serveur de destination : {}",self.destination()));
                    ui.horizontal(|ui|{ui.label("Clé MPS du jeu");ui.add(egui::TextEdit::singleline(&mut self.access_key).password(true));});ui.weak("Cette clé sera incluse dans MpsRepositories.json, conformément au protocole MPS. Le jeton d’administration n’y figure jamais.");
                    let known=self.versions.iter().any(|v|v.pack_key==self.key&&v.pack_version==self.version);
                    if known{ui.colored_label(crate::theme::WARNING,"Cette version existe déjà. Choisir une nouvelle version, ou récupérer son ZIP joueurs ci-dessous.");}
                    let size_ok=self.prepared.as_ref().is_some_and(|(p,_)|self.caps.as_ref().is_some_and(|c|std::fs::metadata(&p.archive).is_ok_and(|m|m.len()<=c.max_upload_bytes)));
                    let publish_allowed=self.caps.as_ref().is_some_and(|c|c.scopes.as_ref().is_none_or(|s|s.iter().any(|x|x=="publish")));
                    if ui.add_enabled(publish_allowed&&size_ok&&!known,crate::theme::primary("Publier cette version sur OpenMPS…")).clicked(){self.confirm_publish=true;}
                    if self.confirm_publish {
                        ui.strong(format!("Destination : {}",self.destination()));
                        ui.label(format!("Pack : {} · version : {}",self.key,self.version));
                        ui.horizontal(|ui|{if ui.button("Confirmer la publication").clicked(){publish=true;self.confirm_publish=false;}
                        if ui.button("Annuler").clicked(){self.confirm_publish=false;}});
                    }
                    if let Some(receipt)=&self.publication{ui.label(format!("Version publiée : {} {} · {} fichiers · {} domaine(s)",receipt.pack_key,receipt.pack_version,receipt.files_count,receipt.domains.len()));ui.label(format!("URL joueurs : {}",receipt.public_url));}
                    }
                    if self.tab == Tab::Packs {
                    ui.heading("Packs distants");
                    ui.add(egui::TextEdit::singleline(&mut self.remote_filter).hint_text("Rechercher un pack ou une version…").desired_width(f32::INFINITY));
                    if self.caps.is_none() { ui.weak("Choisir un serveur puis cliquer sur Actualiser / connecter."); }
                    else if self.versions.is_empty() { ui.weak("Ce serveur ne contient encore aucun pack. Utiliser l’onglet Publier pour en ajouter un."); }
                    ui.separator();ui.heading("4. Récupérer le ZIP joueurs");
                    ui.weak("Le téléchargement vérifie l’empreinte et le contenu du ZIP. Le fichier est enregistré localement ; CPM n’installe rien automatiquement dans Minecraft.");
                    ui.separator();ui.heading("Récupérer une copie éditable");
                    ui.strong(format!("Serveur source : {}",self.destination()));
                    ui.weak("Récupère les configurations et les ressources originales encore présentes sur le serveur. Nécessite un jeton avec le droit recover. Chaque copie est créée dans un nouveau dossier ; aucun pack local n’est remplacé.");
                    ui.horizontal(|ui|{ui.label("Dossier de destination");ui.text_edit_singleline(&mut self.recovery_parent);if ui.button("Choisir un dossier…").clicked(){if let Some(path)=rfd::FileDialog::new().pick_folder(){self.recovery_parent=path.display().to_string();}}});
                    if self.caps.as_ref().is_some_and(|c|!c.editable_download||!c.recovery_allowed){ui.colored_label(crate::theme::WARNING,"Mettre à jour l’API et utiliser un jeton avec read,recover pour récupérer les originaux.");}
                    if let Some(path)=&self.recovered {ui.monospace(path.display().to_string());if ui.add_enabled(can_open,egui::Button::new("Ouvrir la copie récupérée")).on_hover_text("Exporter ou annuler les changements du pack actuel avant d’ouvrir la copie").clicked(){self.open_request=Some(path.clone());}}
                    let filter=self.remote_filter.to_lowercase();
                    for (i,row) in self.versions.iter().enumerate().filter(|(_,r)|format!("{} {}",r.pack_key,r.pack_version).to_lowercase().contains(&filter)){ui.push_id(i,|ui|{ui.vertical(|ui|{ui.label(format!("{} · {} · {} fichiers",row.pack_key,row.pack_version,row.files_count));if ui.add_enabled(row.client_size>0,egui::Button::new("Télécharger le ZIP joueurs…")).clicked(){download=Some(row.clone());}
                if ui.add_enabled(!self.recovery_parent.trim().is_empty()&&self.caps.as_ref().is_some_and(|c|c.editable_download&&c.recovery_allowed),egui::Button::new("Récupérer pour modifier…")).clicked(){recover=Some(row.clone());}});});}
                    }
                });
            });
            if busy {
                let bytes=self.progress.load(Ordering::Relaxed);ui.horizontal(|ui|{ui.spinner();ui.label(&self.status);});
                if self.total>0 {ui.add(egui::ProgressBar::new((bytes as f64/self.total as f64).min(1.0) as f32).show_percentage());if bytes>=self.total{ui.weak("Transfert terminé ; le serveur prépare les fichiers MPS…");}}
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }else{ui.label(&self.status);}
            if let Some(error)=&self.error{egui::ScrollArea::vertical().id_salt("openmps-error").max_height(100.0).show(ui,|ui|{ui.colored_label(crate::theme::ERROR,error);});}
        });
        if !open && busy {
            self.status = "Opération en cours : attendre son résultat avant de fermer".into();
            self.open = true;
        } else {
            self.open = open;
        }
        if connect {
            self.connect_server(ctx);
        }
        if prepare {
            self.total = 0;
            if let Some(pack) = pack {
                let pack = pack.clone();
                let key = self.key.clone();
                let version = self.version.clone();
                self.start(
                    ctx,
                    "Diagnostic et préparation de la copie…",
                    move || {
                        Ok(Completed::Prepared(
                            Arc::new(pack.prepare_openmps(&key, &version, &documents, &catalog)?),
                            revision,
                        ))
                    },
                );
            }
        }
        if publish {
            match (
                self.client(),
                self.prepared.as_ref().map(|(p, _)| p.clone()),
            ) {
                (Ok(client), Some(package)) => {
                    self.progress = Arc::new(AtomicU64::new(0));
                    self.total = std::fs::metadata(&package.archive)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    let progress = self.progress.clone();
                    let key = self.access_key.clone();
                    self.start(ctx, "Envoi de l’archive vers OpenMPS…", move || {
                        Ok(Completed::Published(
                            client.publish(&package, &key, progress)?,
                        ))
                    });
                }
                (Err(e), _) => self.error = Some(e),
                _ => {}
            }
        }
        if let Some(row) = recover {
            match self.client() {
                Ok(client) => {
                    let parent = PathBuf::from(self.recovery_parent.trim());
                    let progress = Arc::new(AtomicU64::new(0));
                    self.progress = progress.clone();
                    self.total = 0;
                    self.start(
                        ctx,
                        "Récupération et vérification du pack éditable…",
                        move || {
                            client
                                .recover(&row, &parent, progress)
                                .map(Completed::Recovered)
                        },
                    );
                }
                Err(e) => self.error = Some(e),
            }
        }
        if let Some(row) = download {
            if let Some(path) = rfd::FileDialog::new()
                .set_title(
                    "Enregistrer le ZIP joueurs (choisir un nouveau nom si le fichier existe)",
                )
                .set_file_name(format!("client-{}-{}.zip", row.pack_key, row.pack_version))
                .add_filter("Archive ZIP", &["zip"])
                .save_file()
            {
                match self.client() {
                    Ok(client) => {
                        self.progress = Arc::new(AtomicU64::new(0));
                        self.total = row.client_size;
                        let progress = self.progress.clone();
                        self.start(
                            ctx,
                            "Téléchargement et vérification du ZIP joueurs…",
                            move || {
                                client.download(&row, &path, progress)?;
                                Ok(Completed::Downloaded(path))
                            },
                        );
                    }
                    Err(e) => self.error = Some(e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tab_tests {
    use super::*;
    fn texts(shape: &egui::epaint::Shape, result: &mut Vec<String>) {
        match shape {
            egui::epaint::Shape::Text(t) => result.push(t.galley.job.text.clone()),
            egui::epaint::Shape::Vec(v) => {
                for s in v {
                    texts(s, result);
                }
            }
            _ => {}
        }
    }
    #[test]
    fn tabs_keep_server_visible_and_show_only_the_selected_workflow() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx);
        let mut app = OpenMps {
            open: true,
            profile_name: "Production".into(),
            endpoint: "https://one.example/".into(),
            ..Default::default()
        };
        for tab in [Tab::Packs, Tab::Publish, Tab::Servers] {
            app.tab = tab;
            let mut labels = Vec::new();
            for _ in 0..2 {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1400.0, 1000.0),
                        )),
                        ..Default::default()
                    },
                    |_| {
                        app.show(
                            &ctx,
                            None,
                            BTreeMap::new(),
                            Arc::new(ParameterCatalog::default()),
                            0,
                            true,
                        )
                    },
                );
                labels.clear();
                for shape in &output.shapes {
                    texts(&shape.shape, &mut labels);
                }
                output.textures_delta.clear();
            }
            assert!(labels
                .iter()
                .any(|s| s.contains("Production — https://one.example/")));
            assert_eq!(
                labels.iter().any(|s| s == "Préparer la publication"),
                tab == Tab::Publish
            );
            assert_eq!(
                labels.iter().any(|s| s == "+ Ajouter un serveur"),
                tab == Tab::Servers
            );
            assert_eq!(
                labels.iter().any(|s| s == "Récupérer une copie éditable"),
                tab == Tab::Packs
            );
            assert!(!app.busy());
        }
    }
}
