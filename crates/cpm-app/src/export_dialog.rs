use cpm_core::{Document, ExportFormat, ExportOptions, Pack, PackMetadata};
use eframe::egui;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};

pub enum Action {
    None,
    Start,
    Review,
    Close,
}

pub struct ExportDialog {
    pub options: ExportOptions,
    pub force: bool,
    destination_text: String,
    pub error: Option<String>,
    worker: Option<Receiver<Result<PathBuf, String>>>,
    review_worker: Option<Receiver<Result<(String, cpm_core::ExportReview), String>>>,
    review: Option<(String, cpm_core::ExportReview)>,
}
impl ExportDialog {
    pub fn new(pack: &Pack, metadata: PackMetadata) -> Self {
        let destination = pack.source().parent().unwrap_or(Path::new(".")).to_owned();
        let output_name = if pack.source().is_dir() {
            pack.source().file_name()
        } else {
            pack.source().file_stem()
        }
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
        let format = if pack.source().is_dir() {
            ExportFormat::Folder
        } else if pack
            .source()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            ExportFormat::Zip
        } else {
            ExportFormat::Dnxpack
        };
        Self {
            destination_text: destination.display().to_string(),
            options: ExportOptions {
                destination,
                output_name,
                format,
                replace_existing: false,
                metadata,
            },
            force: false,
            error: None,
            worker: None,
            review_worker: None,
            review: None,
        }
    }
    pub fn set_destination(&mut self, path: PathBuf) {
        self.destination_text = path.display().to_string();
        self.options.destination = path;
    }
    pub fn busy(&self) -> bool {
        self.worker.is_some() || self.review_worker.is_some()
    }
    pub fn start(
        &mut self,
        pack: Pack,
        documents: BTreeMap<PathBuf, Document>,
        catalog: std::sync::Arc<cpm_core::ParameterCatalog>,
    ) {
        if self.busy() {
            return;
        }
        let signature = self.signature();
        let Some((key, review)) = self.review.take() else {
            self.error = Some("Comparer les modifications avant l’export".into());
            return;
        };
        if key != signature {
            self.error = Some("Les options ont changé ; comparer à nouveau".into());
            return;
        }
        let _ = (pack, documents, catalog);
        let (sender, receiver) = mpsc::channel();
        self.worker = Some(receiver);
        self.error = None;
        std::thread::spawn(move || {
            let _ = sender.send(review.export());
        });
    }
    fn signature(&self) -> String {
        format!("{:?}/{}", self.options, self.force)
    }
    pub fn prepare_review(
        &mut self,
        pack: Pack,
        documents: BTreeMap<PathBuf, Document>,
        catalog: std::sync::Arc<cpm_core::ParameterCatalog>,
    ) {
        if self.busy() {
            return;
        }
        let options = self.options.clone();
        let key = self.signature();
        let force = self.force;
        self.review = None;
        self.error = None;
        let (tx, rx) = mpsc::channel();
        self.review_worker = Some(rx);
        std::thread::spawn(move || {
            let result = pack
                .review_export(&options, &documents, &catalog, force)
                .map(|r| (key, r));
            let _ = tx.send(result);
        });
    }
    pub fn poll(&mut self) -> Option<Result<PathBuf, String>> {
        let result = match self.worker.as_ref()?.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err("Le traitement d'export a été interrompu".into())
            }
        };
        self.worker = None;
        Some(result)
    }
    pub fn show(&mut self, ctx: &egui::Context) -> Action {
        let mut action = Action::None;
        if let Some(rx) = &self.review_worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.review_worker = None;
                    match result {
                        Ok(review) => self.review = Some(review),
                        Err(e) => self.error = Some(e),
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.review_worker = None;
                    self.error = Some("Comparaison interrompue".into());
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        if self
            .review
            .as_ref()
            .is_some_and(|(key, _)| *key != self.signature())
        {
            self.review = None;
        }
        let busy = self.busy();
        egui::Window::new("Exporter le pack")
            .id(egui::Id::unique("export-pack-window"))
            .collapsible(false)
            .resizable(true)
            .default_width(650.0)
            .max_height(680.0)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.add_enabled_ui(!busy, |ui| {
                    if self.force {
                        ui.colored_label(egui::Color32::YELLOW, "Export forcé : les erreurs de configuration sont ignorées. Le pack peut ne pas fonctionner en jeu.");
                    }
                    ui.label("Dossier de destination");
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut self.destination_text)
                                    .desired_width(410.0),
                            )
                            .changed()
                        {
                            self.options.destination = PathBuf::from(&self.destination_text);
                        }
                        if ui.button("Parcourir…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Choisir le dossier de destination")
                                .set_directory(&self.options.destination)
                                .pick_folder()
                            {
                                self.destination_text = path.display().to_string();
                                self.options.destination = path;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Format");
                        for format in [
                            ExportFormat::Folder,
                            ExportFormat::Zip,
                            ExportFormat::Dnxpack,
                        ] {
                            ui.selectable_value(&mut self.options.format, format, format.label());
                        }
                    });
                    ui.label("Nom du fichier ou dossier de sortie");
                    ui.text_edit_singleline(&mut self.options.output_name);
                    ui.checkbox(
                        &mut self.options.replace_existing,
                        "Remplacer la sortie si elle existe déjà (y compris le pack original)",
                    );
                    ui.small("Le remplacement concerne exactement le chemin de sortie affiché ci-dessous.");
                    ui.separator();
                    ui.label("Nom interne du pack (PackName)");
                    ui.text_edit_singleline(&mut self.options.metadata.name);
                    ui.small("Ce nom peut aussi être référencé par d'autres packs.");
                    ui.label("Version du pack");
                    ui.text_edit_singleline(&mut self.options.metadata.version);
                    ui.label("Description");
                    ui.text_edit_singleline(&mut self.options.metadata.description);
                    ui.separator();
                    ui.label("Chemin de sortie");
                    match self.options.target() {
                        Ok(target) => {
                            ui.monospace(target.display().to_string());
                        }
                        Err(error) => {
                            ui.colored_label(egui::Color32::YELLOW, error);
                        }
                    }
                    ui.separator();
                    if ui.button("Comparer les modifications avant export").clicked() {action=Action::Review;}
                    if let Some((_,review))=&self.review {
                        ui.label(format!("{} fichier(s) ajouté(s), supprimé(s) ou modifié(s)",review.changes.len()));
                        ui.weak("La copie comparée est figée. Modifier une option impose une nouvelle comparaison.");
                        egui::ScrollArea::vertical().id_salt("export-changes").max_height(220.0).show(ui,|ui|{
                            for change in &review.changes {egui::CollapsingHeader::new(format!("{} · {}",change.kind.label(),change.path.display())).id_salt(&change.path).show(ui,|ui|{
                                if change.text_only {ui.label("Texte, commentaires ou structure modifiés ; valeurs identiques ou texte non analysable.");}
                                for value in &change.parameters {ui.label(&value.parameter);ui.horizontal_wrapped(|ui|{ui.colored_label(egui::Color32::LIGHT_RED,format!("Avant : {}",value.before.as_deref().unwrap_or("(absent)")));ui.colored_label(egui::Color32::LIGHT_GREEN,format!("Après : {}",value.after.as_deref().unwrap_or("(absent)")));});}
                            });}
                        });
                    }
                    ui.horizontal(|ui| {
                        if ui.add_enabled(self.review.as_ref().is_some_and(|(key,_)|*key==self.signature()),egui::Button::new("Exporter la copie comparée")).clicked() {
                            action = Action::Start;
                        }
                        if ui.button("Fermer").clicked() {
                            action = Action::Close;
                        }
                    });
                });
                if busy {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(if self.review_worker.is_some() {"Préparation et comparaison de la copie…"} else {"Export en cours…"});
                    });
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                }
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::LIGHT_RED, error);
                }
            });
        action
    }
}

#[cfg(test)]
mod review_tests {
    use super::*;
    #[test]
    fn export_requires_review_and_options_changes_invalidate_it() {
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("pack_info.dynx"),
            "PackName: Demo\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        let pack = Pack::open(root.path()).unwrap();
        let catalog = std::sync::Arc::new(cpm_core::ParameterCatalog::builtin());
        let mut dialog = ExportDialog::new(
            &pack,
            PackMetadata {
                name: "Demo".into(),
                ..Default::default()
            },
        );
        dialog.options.output_name = "reviewed".into();
        dialog.set_destination(target.path().into());
        dialog.start(pack.clone(), BTreeMap::new(), catalog.clone());
        assert!(dialog.error.is_some());
        assert!(!dialog.busy());
        let review = pack
            .review_export(&dialog.options, &BTreeMap::new(), &catalog, false)
            .unwrap();
        dialog.review = Some((dialog.signature(), review));
        dialog.options.metadata.version = "2".into();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.show(ui.ctx());
        });
        output.textures_delta.clear();
        assert!(dialog.review.is_none());
        assert!(!target.path().join("reviewed").exists());
    }
}
