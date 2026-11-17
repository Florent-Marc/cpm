use cpm_core::{Document, ExportFormat, ExportOptions, Pack, PackMetadata};
use eframe::egui;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};

pub enum Action {
    None,
    Start,
    Close,
}

pub struct ExportDialog {
    pub options: ExportOptions,
    destination_text: String,
    pub error: Option<String>,
    worker: Option<Receiver<Result<PathBuf, String>>>,
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
            error: None,
            worker: None,
        }
    }
    pub fn busy(&self) -> bool {
        self.worker.is_some()
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
        let options = self.options.clone();
        let (sender, receiver) = mpsc::channel();
        self.worker = Some(receiver);
        self.error = None;
        std::thread::spawn(move || {
            let result = pack.export_with_catalog(&options, &documents, &catalog);
            let _ = sender.send(result);
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
        let busy = self.busy();
        egui::Window::new("Exporter le pack")
            .id(egui::Id::unique("export-pack-window"))
            .collapsible(false)
            .resizable(true)
            .default_width(580.0)
            .show(ctx, |ui| {
                ui.add_enabled_ui(!busy, |ui| {
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
                    ui.horizontal(|ui| {
                        if ui.button("Exporter").clicked() {
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
                        ui.label("Export en cours…");
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
