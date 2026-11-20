pub mod client;
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
enum Completed {
    Connected(Capabilities, Vec<Version>),
    Prepared(Arc<PreparedOpenMps>, u64),
    Published(Publication),
    Downloaded(PathBuf),
}
#[derive(Default)]
pub struct OpenMps {
    pub open: bool,
    profiles: Vec<Profile>,
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
            Ok(profiles) => self.profiles = profiles,
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
    ) {
        self.poll();
        if !self.open {
            return;
        }
        if self.prepared.as_ref().is_some_and(|(_, r)| *r != revision) {
            self.prepared = None;
            self.status = "Le pack a changé ; préparer à nouveau la copie avant publication".into();
        }
        let busy = self.busy();
        let mut open = true;
        let mut connect = false;
        let mut prepare = false;
        let mut publish = false;
        let mut download = None;
        egui::Window::new("OpenMPS — publier et récupérer le pack joueurs").open(&mut open).default_size([780.0,680.0]).min_size([560.0,440.0]).collapsible(false).show(ctx,|ui|{
            ui.label("Cette fenêtre est facultative. File → Exporter fonctionne toujours sans serveur ni connexion.");
            egui::ScrollArea::vertical().id_salt("openmps-window").max_height((ui.available_height()-95.0).max(200.0)).show(ui,|ui|{
                ui.add_enabled_ui(!busy,|ui|{
                    ui.heading("1. Connexion au serveur");
                    let mut selected=None;
                    egui::ComboBox::from_id_salt("openmps-profiles").selected_text("Profils enregistrés…").show_ui(ui,|ui|{for (i,profile) in self.profiles.iter().enumerate(){if ui.selectable_label(false,&profile.name).clicked(){selected=Some(i);}}});
                    if let Some(i)=selected {self.profile_name=self.profiles[i].name.clone();self.endpoint=self.profiles[i].url.clone();self.token.clear();self.caps=None;self.versions.clear();}
                    ui.horizontal(|ui|{ui.label("Nom du profil");ui.text_edit_singleline(&mut self.profile_name);});
                    ui.horizontal(|ui|{ui.label("URL du serveur");if ui.add(egui::TextEdit::singleline(&mut self.endpoint).hint_text("https://mps.exemple.fr/").desired_width(f32::INFINITY)).changed(){self.caps=None;self.versions.clear();}});
                    ui.horizontal(|ui|{ui.label("Jeton d’administration");if ui.add(egui::TextEdit::singleline(&mut self.token).password(true).desired_width(f32::INFINITY)).changed(){self.caps=None;self.versions.clear();}});
                    ui.weak("Le jeton reste en mémoire pour cette session et n’est pas enregistré dans le profil. Il est différent de la clé MPS du jeu.");
                    ui.horizontal(|ui|{
                        if ui.button("Tester la connexion / actualiser").clicked(){connect=true;}
                        if ui.button("Enregistrer le profil").clicked(){
                            let placeholder=format!("omps_{}","x".repeat(43));
                            match Client::new(&self.endpoint,&placeholder).and_then(|_|Self::profiles_path()) {
                                Ok(path) if !self.profile_name.trim().is_empty()=>{self.profiles.retain(|p|p.name!=self.profile_name);self.profiles.push(Profile{name:self.profile_name.clone(),url:self.endpoint.clone()});if let Err(e)=client::save_profiles(&path,&self.profiles){self.error=Some(e);}else{self.status="Profil enregistré sans jeton".into();}}
                                Ok(_)=>self.error=Some("Donner un nom au profil".into()),Err(e)=>self.error=Some(e),
                            }
                        }
                    });
                    ui.collapsing("Configurer OpenMPS une première fois",|ui|{ui.label("Installer l’API d’administration v1 sur votre serveur, définir SERVER_PUBLIC_URL pour les joueurs, puis créer un jeton en ligne de commande :");ui.monospace("php tools/api_token.php create CPM");ui.label("Coller ici le jeton affiché. Ne pas utiliser le mot de passe du tableau de bord ni la clé MPS du jeu.");});
                    if let Some(caps)=&self.caps{ui.label(format!("API v{} · MPS {} · limite {:.0} Mio",caps.api_version,caps.mps_version,caps.max_upload_bytes as f64/1048576.0));ui.label(format!("URL utilisée par les joueurs : {}",caps.public_url));}
                    ui.separator();ui.heading("2. Préparer une copie compatible");
                    ui.label("Le diagnostic contrôle le pack et prépare une copie avec les ressources sous assets/. Le pack de travail et ses exports classiques ne sont pas modifiés.");
                    ui.horizontal(|ui|{ui.label("Identifiant OpenMPS / PackName publié");if ui.text_edit_singleline(&mut self.key).changed(){self.prepared=None;self.publication=None;}});
                    ui.horizontal(|ui|{ui.label("Version à publier");if ui.text_edit_singleline(&mut self.version).changed(){self.prepared=None;self.publication=None;}});
                    ui.weak("L’identifiant et les références internes sont adaptés dans la copie. Une version déjà présente sur le serveur ne sera pas écrasée.");
                    if ui.add_enabled(pack.is_some(),egui::Button::new("Vérifier et préparer la publication")).clicked(){prepare=true;}
                    if pack.is_none(){ui.weak("Ouvrir un pack pour publier. La liste distante reste consultable sans pack local.");}
                    if let Some((package,_))=&self.prepared {
                        let report=&package.report;let size=std::fs::metadata(&package.archive).map(|m|m.len()).unwrap_or(0);
                        ui.label(format!("{} {} · {} fichiers · {:.1} Mio",package.key,package.version,report.files,size as f64/1048576.0));
                        ui.collapsing(format!("Ressources déplacées dans la copie ({})",report.moved.len()),|ui|{for (from,to) in &report.moved{ui.label(format!("{} → {}",from.display(),to.display()));}});
                        if !report.skipped.is_empty(){ui.collapsing("Fichiers de documentation ou anciens fichiers MPS exclus",|ui|{for p in &report.skipped{ui.label(p.display().to_string());}});}
                        for warning in &report.warnings{ui.colored_label(egui::Color32::YELLOW,warning);}
                        if self.caps.as_ref().is_some_and(|caps|size>caps.max_upload_bytes){ui.colored_label(egui::Color32::LIGHT_RED,"L’archive dépasse la limite annoncée par le serveur.");}
                    }
                    ui.separator();ui.heading("3. Publier la version");
                    ui.horizontal(|ui|{ui.label("Clé MPS du jeu");ui.add(egui::TextEdit::singleline(&mut self.access_key).password(true));});ui.weak("Cette clé sera incluse dans MpsRepositories.json, conformément au protocole MPS. Le jeton d’administration n’y figure jamais.");
                    let known=self.versions.iter().any(|v|v.pack_key==self.key&&v.pack_version==self.version);
                    if known{ui.colored_label(egui::Color32::YELLOW,"Cette version existe déjà. Choisir une nouvelle version, ou récupérer son ZIP joueurs ci-dessous.");}
                    let size_ok=self.prepared.as_ref().is_some_and(|(p,_)|self.caps.as_ref().is_some_and(|c|std::fs::metadata(&p.archive).is_ok_and(|m|m.len()<=c.max_upload_bytes)));
                    if ui.add_enabled(self.caps.is_some()&&size_ok&&!known,egui::Button::new("Publier cette version sur OpenMPS")).clicked(){publish=true;}
                    if let Some(receipt)=&self.publication{ui.label(format!("Version publiée : {} {} · {} fichiers · {} domaine(s)",receipt.pack_key,receipt.pack_version,receipt.files_count,receipt.domains.len()));ui.label(format!("URL joueurs : {}",receipt.public_url));}
                    ui.separator();ui.heading("4. Récupérer le ZIP joueurs");
                    ui.weak("Le téléchargement vérifie l’empreinte et le contenu du ZIP. Le fichier est enregistré localement ; CPM n’installe rien automatiquement dans Minecraft.");
                    for (i,row) in self.versions.iter().enumerate(){ui.push_id(i,|ui|{ui.horizontal(|ui|{ui.label(format!("{} · {} · {} fichiers",row.pack_key,row.pack_version,row.files_count));if ui.add_enabled(row.client_size>0,egui::Button::new("Télécharger le ZIP joueurs…")).clicked(){download=Some(row.clone());}});});}
                });
            });
            if busy {
                let bytes=self.progress.load(Ordering::Relaxed);ui.horizontal(|ui|{ui.spinner();ui.label(&self.status);});
                if self.total>0 {ui.add(egui::ProgressBar::new((bytes as f64/self.total as f64).min(1.0) as f32).show_percentage());if bytes>=self.total{ui.weak("Transfert terminé ; le serveur prépare les fichiers MPS…");}}
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }else{ui.label(&self.status);}
            if let Some(error)=&self.error{egui::ScrollArea::vertical().id_salt("openmps-error").max_height(100.0).show(ui,|ui|{ui.colored_label(egui::Color32::LIGHT_RED,error);});}
        });
        if !open && busy {
            self.status = "Opération en cours : attendre son résultat avant de fermer".into();
            self.open = true;
        } else {
            self.open = open;
        }
        if connect {
            self.total = 0;
            match self.client() {
                Ok(client) => self.start(ctx, "Connexion et lecture des versions…", move || {
                    let (caps, rows) = client.connect()?;
                    Ok(Completed::Connected(caps, rows))
                }),
                Err(e) => self.error = Some(e),
            }
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
