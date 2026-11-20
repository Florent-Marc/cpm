#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod creation;
mod editor;
mod export_dialog;
mod model_browser;
mod navigation;
mod openmps;
mod parameter_input;
mod preview;
mod properties_editor;
mod recent;
mod report;
mod search;
mod update;

use cpm_core::Pack;
use editor::Editor;
use eframe::egui;
use std::collections::BTreeMap;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        depth_buffer: 24,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 740.0])
            .with_min_inner_size([760.0, 480.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/cpm.png"))
                    .expect("logo CPM valide"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "CPM — Éditeur de configuration",
        options,
        Box::new(|cc| {
            let mut app = CpmApp {
                gl: cc.gl.clone(),
                ..Default::default()
            };
            app.load_properties();
            app.load_recent();
            app.openmps.load_profiles();
            app.updater.check();
            Ok(Box::new(app))
        }),
    )
}

#[derive(Default)]
struct CpmApp {
    gl: Option<std::sync::Arc<eframe::glow::Context>>,
    openmps: openmps::OpenMps,
    preview: Option<preview::Preview>,
    preview_config: std::path::PathBuf,
    preview_source: String,
    preview_signature: u64,
    preview_due: Option<std::time::Instant>,
    search: search::Search,
    report: Option<report::Report>,
    model_browser: Option<model_browser::Browser>,
    creation: Option<creation::Creation>,
    manager: Option<creation::Manager>,
    draft_pack: Option<tempfile::TempDir>,
    draft_name: String,
    draft_file: bool,
    recent: recent::RecentPacks,
    catalog: std::sync::Arc<cpm_core::ParameterCatalog>,
    properties_editor: Option<properties_editor::PropertiesEditor>,
    editor: Option<Editor>,
    pack: Option<Pack>,
    cached: BTreeMap<usize, Editor>,
    selected_config: Option<usize>,
    open_path: String,
    standalone_path: Option<std::path::PathBuf>,
    selected_section: Option<usize>,
    pending_scroll: Option<Option<usize>>,
    status: String,
    error: Option<String>,
    confirm_close: bool,
    allow_close: bool,
    export_dialog: Option<export_dialog::ExportDialog>,
    validation: BTreeMap<usize, Vec<String>>,
    validation_worker: Option<std::sync::mpsc::Receiver<(usize, Vec<String>)>>,
    updater: update::Updater,
    restarting: bool,
}

impl CpmApp {
    fn load_recent(&mut self) {
        match std::env::current_exe()
            .map_err(|e| e.to_string())
            .and_then(|p| {
                p.parent()
                    .map(|p| p.join("recent-packs.json"))
                    .ok_or("Dossier introuvable".into())
            }) {
            Ok(path) => {
                let (recent, warning) = recent::RecentPacks::load(path);
                self.recent = recent;
                if let Some(warning) = warning {
                    self.error = Some(match self.error.take() {
                        Some(error) => format!("{error}\n{warning}"),
                        None => warning,
                    });
                }
            }
            Err(error) => self.error = Some(error),
        }
    }
    fn remember_pack(&mut self, path: &std::path::Path) {
        if let Err(error) = self.recent.remember(path) {
            self.error = Some(error);
        }
    }
    fn properties_directory() -> Result<std::path::PathBuf, String> {
        Ok(std::env::current_exe()
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("Dossier introuvable")?
            .join("properties"))
    }
    fn load_properties(&mut self) {
        match Self::properties_directory()
            .and_then(|directory| (cpm_core::CatalogStore { directory }).load())
        {
            Ok(loaded) => {
                self.apply_catalog(loaded.catalog);
                self.error = (!loaded.warnings.is_empty()).then(|| loaded.warnings.join("\n"));
            }
            Err(error) => self.error = Some(format!("Catalogue : {error}")),
        }
    }
    fn apply_catalog(&mut self, catalog: cpm_core::ParameterCatalog) {
        self.catalog = std::sync::Arc::new(catalog);
        if let Some(editor) = &mut self.editor {
            editor.catalog = self.catalog.clone();
        }
        for editor in self.cached.values_mut() {
            editor.catalog = self.catalog.clone();
        }
        self.start_validation();
    }
    fn dirty(&self) -> bool {
        self.draft_pack.is_some()
            || self.draft_file
            || self.pack.as_ref().is_some_and(Pack::has_structure_changes)
            || self.editor.as_ref().is_some_and(Editor::is_dirty)
            || self.cached.values().any(Editor::is_dirty)
    }

    #[cfg(test)]
    fn open(&mut self) {
        let path = std::path::PathBuf::from(&self.open_path);
        self.open_selection(&path);
    }

    fn open_selection(&mut self, path: &std::path::Path) {
        if self.dirty() {
            self.error = Some(
                "Enregistre ou annule les modifications avant d'ouvrir un autre fichier ou pack."
                    .into(),
            );
            return;
        }
        self.preview = None;
        self.model_browser = None;
        self.search = search::Search::default();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("dynx"))
        {
            match Editor::open(path) {
                Ok(mut editor) => {
                    editor.catalog = self.catalog.clone();
                    self.editor = Some(editor);
                    self.pack = None;
                    self.draft_pack = None;
                    self.draft_file = false;
                    self.validation.clear();
                    self.validation_worker = None;
                    self.cached.clear();
                    self.selected_config = None;
                    self.selected_section = None;
                    self.pending_scroll = None;
                    self.standalone_path = Some(path.to_owned());
                    self.open_path = path.display().to_string();
                    self.error = None;
                    self.status = "Configuration ouverte".into();
                }
                Err(error) => self.error = Some(error),
            }
            return;
        }
        match Pack::open(path) {
            Ok(pack) => {
                self.pack = Some(pack);
                self.draft_pack = None;
                self.draft_file = false;
                self.standalone_path = None;
                self.open_path = path.display().to_string();
                self.editor = None;
                self.cached.clear();
                self.selected_config = None;
                self.status = "Pack ouvert".into();
                self.start_validation();
                self.switch_config(0);
                self.remember_pack(path);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn start_validation(&mut self) {
        self.validation.clear();
        let Some(pack) = self.pack.clone() else {
            return;
        };
        let catalog = self.catalog.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        self.validation_worker = Some(receiver);
        std::thread::spawn(move || {
            for (index, path) in pack.configs().iter().enumerate() {
                let issues = match pack.read_config(index) {
                    Ok(document) => catalog.errors(path, &document),
                    Err(error) => vec![error],
                };
                if sender.send((index, issues)).is_err() {
                    break;
                }
            }
        });
    }

    fn poll_validation(&mut self) {
        let mut finished = false;
        if let Some(receiver) = &self.validation_worker {
            loop {
                match receiver.try_recv() {
                    Ok((index, issues)) => {
                        self.validation.insert(index, issues);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        finished = true;
                        break;
                    }
                }
            }
        }
        if finished {
            self.validation_worker = None;
        }
    }

    fn issues_for(&self, index: usize) -> Vec<String> {
        if self.selected_config == Some(index) {
            if let Some(editor) = &self.editor {
                return editor.issues();
            }
        }
        if let Some(editor) = self.cached.get(&index) {
            return editor.issues();
        }
        self.validation.get(&index).cloned().unwrap_or_default()
    }

    fn export_current_file(&mut self) {
        let Some(editor) = &mut self.editor else {
            return;
        };
        let issues = editor.issues();
        if !issues.is_empty() {
            self.error = Some(format!("Export bloqué :\n{}", issues.join("\n")));
            return;
        }
        let filename = if self.draft_file {
            editor
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            format!(
                "{}_edited.dynx",
                editor
                    .path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
            )
        };
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Exporter la configuration")
            .set_file_name(filename)
            .add_filter("Configuration DynamX", &["dynx"])
            .save_file()
        {
            match editor.save_copy(&path) {
                Ok(()) => {
                    if self.draft_file {
                        editor.path = path.clone();
                        self.standalone_path = Some(path.clone());
                        self.open_path = path.display().to_string();
                    }
                    self.draft_file = false;
                    self.status = format!("Fichier exporté : {}", path.display());
                    self.error = None;
                }
                Err(error) => self.error = Some(error),
            }
        }
    }

    fn documents(&self) -> BTreeMap<std::path::PathBuf, cpm_core::Document> {
        let mut documents = BTreeMap::new();
        if let Some(pack) = &self.pack {
            for (index, editor) in &self.cached {
                documents.insert(pack.configs()[*index].clone(), editor.document.clone());
            }
            if let (Some(index), Some(editor)) = (self.selected_config, &self.editor) {
                documents.insert(pack.configs()[index].clone(), editor.document.clone());
            }
        }
        documents
    }

    fn prepare_export(&mut self) {
        if let Some(pack) = &self.pack {
            self.report = Some(report::Report::new(
                pack.clone(),
                self.documents(),
                self.catalog.clone(),
                true,
            ));
        }
    }
    fn prepare_export_options(&mut self) {
        if let Some(pack) = &self.pack {
            let documents = self.documents();
            let metadata = documents
                .get(std::path::Path::new("pack_info.dynx"))
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| pack.read_config(0));
            match metadata {
                Ok(document) => {
                    let mut dialog = export_dialog::ExportDialog::new(
                        pack,
                        cpm_core::PackMetadata::from_document(&document),
                    );
                    if self.draft_pack.is_some() {
                        dialog.options.output_name = self.draft_name.clone();
                        dialog.set_destination(
                            std::env::current_dir()
                                .unwrap_or_else(|_| std::path::PathBuf::from(".")),
                        );
                    }
                    self.export_dialog = Some(dialog);
                }
                Err(error) => self.error = Some(error),
            }
        }
    }

    fn adopt_export(&mut self, path: &std::path::Path) {
        match Pack::open(path) {
            Ok(pack) => {
                self.pack = Some(pack);
                self.draft_pack = None;
                self.draft_file = false;
                self.editor = None;
                self.cached.clear();
                self.selected_config = None;
                self.selected_section = None;
                self.pending_scroll = None;
                self.open_path = path.display().to_string();
                self.standalone_path = None;
                self.start_validation();
                self.switch_config(0);
                self.remember_pack(path);
                self.status = format!("Pack exporté : {}", path.display());
            }
            Err(error) => {
                self.error = Some(format!(
                    "Export terminé, mais réouverture impossible : {error}"
                ))
            }
        }
    }

    fn create_document(&mut self, created: creation::Created) -> Result<(), String> {
        match created {
            creation::Created::Bundle {
                primary,
                configurations,
                assets,
                new_pack,
            } => {
                if new_pack.is_some() && self.dirty() {
                    return Err(
                        "Enregistrer ou annuler les changements avant de créer un nouveau pack"
                            .into(),
                    );
                }
                let mut temporary = None;
                let mut candidate = if let Some((info, _)) = &new_pack {
                    let directory = tempfile::Builder::new()
                        .prefix("cpm-guided-pack-")
                        .tempdir()
                        .map_err(|e| e.to_string())?;
                    std::fs::write(directory.path().join("pack_info.dynx"), info.source())
                        .map_err(|e| e.to_string())?;
                    let pack = Pack::open(directory.path())?;
                    temporary = Some(directory);
                    pack
                } else {
                    self.pack
                        .clone()
                        .ok_or("Ouvrir un pack pour importer des ressources")?
                };
                candidate.add_bundle(configurations, assets)?;
                let index = candidate
                    .configs()
                    .iter()
                    .position(|p| *p == primary)
                    .ok_or("Configuration principale absente du lot préparé")?;
                if let Some((_, name)) = new_pack {
                    self.cached.clear();
                    self.editor = None;
                    self.selected_config = None;
                    self.standalone_path = None;
                    self.draft_name = name.clone();
                    self.open_path = format!("Nouveau pack : {name}");
                    self.draft_pack = temporary;
                    self.draft_file = false;
                }
                self.pack = Some(candidate);
                self.preview = None;
                self.model_browser = None;
                self.search = search::Search::default();
                self.start_validation();
                self.switch_config(index);
                self.status =
                    "Objet et ressources ajoutés — vérifier l’aperçu 3D, puis File → Exporter"
                        .into();
            }
            creation::Created::Pack { document, name } => {
                if self.dirty() {
                    return Err(
                        "Enregistrer ou annuler les modifications avant de créer un pack".into(),
                    );
                }
                let temporary = tempfile::Builder::new()
                    .prefix("cpm-new-pack-")
                    .tempdir()
                    .map_err(|e| e.to_string())?;
                std::fs::write(temporary.path().join("pack_info.dynx"), document.source())
                    .map_err(|e| e.to_string())?;
                let pack = Pack::open(temporary.path())?;
                self.pack = Some(pack);
                self.cached.clear();
                self.editor = None;
                self.selected_config = None;
                self.standalone_path = None;
                self.draft_name = name.clone();
                self.open_path = format!("Nouveau pack : {name}");
                self.draft_pack = Some(temporary);
                self.draft_file = false;
                self.preview = None;
                self.start_validation();
                self.switch_config(0);
                self.status = "Nouveau pack créé — File → Exporter pour l’enregistrer".into();
            }
            creation::Created::Configuration { path, document } => {
                if let Some(pack) = &mut self.pack {
                    let index = pack.add_configuration(&path, document)?;
                    self.start_validation();
                    self.switch_config(index);
                    self.status =
                        format!("Configuration ajoutée : {} — à exporter", path.display());
                } else {
                    if self.dirty() {
                        return Err("Enregistrer ou annuler les modifications avant de créer une configuration".into());
                    }
                    let mut editor = Editor::from_document(document, &path);
                    editor.catalog = self.catalog.clone();
                    self.editor = Some(editor);
                    self.standalone_path = Some(path.clone());
                    self.open_path = path.display().to_string();
                    self.cached.clear();
                    self.selected_config = None;
                    self.selected_section = None;
                    self.pending_scroll = None;
                    self.validation.clear();
                    self.validation_worker = None;
                    self.draft_file = true;
                    self.preview = None;
                    self.status =
                        "Nouvelle configuration créée — File → Exporter pour l’enregistrer".into();
                }
            }
        }
        self.error = None;
        Ok(())
    }
    fn import_configuration(&mut self, path: &std::path::Path) -> Result<(), String> {
        use std::io::Read as _;
        if self.pack.is_none() {
            return Err("Ouvrir ou créer un pack avant d’importer une configuration".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(8 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 8 * 1024 * 1024 {
            return Err("Configuration trop volumineuse (maximum 8 Mio)".into());
        }
        let document =
            cpm_core::Document::read_from(bytes.as_slice()).map_err(|e| e.to_string())?;
        let filename = path.file_name().ok_or("Nom de fichier invalide")?;
        self.create_document(creation::Created::Configuration {
            path: filename.into(),
            document,
        })
    }

    fn delete_configuration(&mut self, path: &std::path::Path) -> Result<(), String> {
        let pack = self.pack.as_mut().ok_or("Aucun pack ouvert")?;
        let old_paths = pack.configs().to_vec();
        let selected = self.selected_config.and_then(|i| old_paths.get(i)).cloned();
        pack.remove_configuration(path)?;
        self.reindex_editors(old_paths, selected);
        self.status = format!("Configuration supprimée : {} — à exporter", path.display());
        Ok(())
    }

    fn reindex_editors(
        &mut self,
        old_paths: Vec<std::path::PathBuf>,
        selected: Option<std::path::PathBuf>,
    ) {
        let mut editors: BTreeMap<_, _> = std::mem::take(&mut self.cached)
            .into_iter()
            .map(|(index, editor)| (old_paths[index].clone(), editor))
            .collect();
        if let (Some(selected), Some(editor)) = (&selected, self.editor.take()) {
            editors.insert(selected.clone(), editor);
        }
        self.selected_config = None;
        let pack = self.pack.as_ref().expect("pack ouvert");
        for (index, path) in pack.configs().iter().enumerate() {
            if let Some(editor) = editors.remove(path) {
                self.cached.insert(index, editor);
            }
        }
        let next = selected
            .and_then(|p| pack.configs().iter().position(|path| *path == p))
            .unwrap_or(0);
        self.start_validation();
        self.switch_config(next);
    }
    fn undo_structure(&mut self) -> Result<(), String> {
        let old = self.pack.as_ref().ok_or("Aucun pack ouvert")?;
        let replacement = Pack::open(old.source())?;
        let old_paths = old.configs().to_vec();
        let selected = self.selected_config.and_then(|i| old_paths.get(i)).cloned();
        self.pack = Some(replacement);
        self.reindex_editors(old_paths, selected);
        self.status = "Ajouts, suppressions et ressources importées annulés".into();
        Ok(())
    }
    fn discard_draft(&mut self) {
        self.pack = None;
        self.editor = None;
        self.cached.clear();
        self.selected_config = None;
        self.standalone_path = None;
        self.open_path.clear();
        self.validation.clear();
        self.validation_worker = None;
        self.selected_section = None;
        self.pending_scroll = None;
        self.preview = None;
        self.manager = None;
        self.draft_pack = None;
        self.draft_file = false;
        self.error = None;
        self.status = "Brouillon abandonné".into();
    }

    fn modal(&self) -> bool {
        if self.openmps.open || self.openmps.busy() {
            return true;
        }
        self.report.is_some()
            || self.export_dialog.is_some()
            || self.properties_editor.is_some()
            || self.creation.is_some()
            || self.manager.is_some()
            || self.confirm_close
            || self.updater.busy()
            || self.restarting
            || self
                .editor
                .as_ref()
                .is_some_and(|e| e.custom_parameter.is_some())
    }
    fn scene_revision(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        self.open_path.hash(&mut hash);
        for editor in self.cached.values().chain(self.editor.iter()) {
            editor.path.hash(&mut hash);
            editor.document.source().hash(&mut hash);
        }
        self.pack.as_ref().map(|p| p.configs()).hash(&mut hash);
        hash.finish()
    }
    fn request_for(&self, path: &std::path::Path) -> Result<cpm_core::ObjRequest, String> {
        let editor = self
            .editor
            .iter()
            .chain(self.cached.values())
            .find(|e| e.path == path)
            .ok_or("Configuration de l’aperçu fermée ou supprimée")?;
        cpm_core::ObjRequest::resolve(self.pack.as_ref(), path, &editor.document)
    }
    fn switch_config(&mut self, index: usize) {
        if self.selected_config == Some(index) {
            return;
        }
        let Some(pack) = &self.pack else {
            return;
        };
        let Some(path) = pack.configs().get(index) else {
            return;
        };
        let mut next = if let Some(editor) = self.cached.remove(&index) {
            editor
        } else {
            match pack.read_config(index) {
                Ok(document) => Editor::from_document(document, path),
                Err(error) => {
                    self.error = Some(error);
                    return;
                }
            }
        };
        if let (Some(previous), Some(mut editor)) = (self.selected_config, self.editor.take()) {
            editor.checkpoint();
            self.cached.insert(previous, editor);
        }
        next.catalog = self.catalog.clone();
        self.editor = Some(next);
        self.selected_config = Some(index);
        self.selected_section = None;
        self.pending_scroll = None;
        self.error = None;
    }
}

impl eframe::App for CpmApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

impl CpmApp {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if let Some(result) = self.updater.poll() {
            match result.and_then(update::install) {
                Ok(()) => {
                    self.allow_close = true;
                    self.restarting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(error) => {
                    self.updater.message = error.clone();
                    self.error = Some(error);
                }
            }
        }
        if self.updater.checking() || self.updater.busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_secs(60));
        }
        if self.openmps.busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        }
        self.poll_validation();
        if self.validation_worker.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        let exporting = self
            .export_dialog
            .as_ref()
            .is_some_and(export_dialog::ExportDialog::busy);
        let updating = self.updater.busy();
        if (exporting || updating) && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if !exporting
            && !updating
            && ctx.input(|i| i.viewport().close_requested())
            && self.dirty()
            && !self.allow_close
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        if self.export_dialog.is_some()
            || self.properties_editor.is_some()
            || self.creation.is_some()
            || self.manager.is_some()
            || self.report.is_some()
            || self.openmps.open
            || self.openmps.busy()
            || updating
            || self.restarting
        {
            ui.disable();
        }
        if !self.modal() && !self.search.open && self.model_browser.is_none() {
            let redo = ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z)
                    || i.consume_key(egui::Modifiers::CTRL, egui::Key::Y)
            });
            let undo = ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Z));
            if let Some(editor) = &mut self.editor {
                if undo {
                    editor.undo();
                } else if redo {
                    editor.redo();
                }
                if undo || redo {
                    self.pending_scroll = None;
                    self.selected_section = None;
                }
            }
        }
        egui::Panel::top("toolbar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add_enabled(self.pack.is_some(),egui::Button::new("Publier sur OpenMPS…")).clicked(){
                        let info=self.documents().get(std::path::Path::new("pack_info.dynx")).cloned().or_else(||self.pack.as_ref().and_then(|p|p.read_config(0).ok()));
                        self.openmps.begin(info.as_ref().map(cpm_core::PackMetadata::from_document));ui.close();
                    }

                    if ui.button("Créer").clicked(){self.creation=Some(creation::Creation::new(self.pack.is_some()));ui.close();}
                    ui.add_enabled_ui(!self.dirty(), |ui| {
                        ui.menu_button("Ouvrir récent", |ui| {
                            if self.recent.paths.is_empty() { ui.label("Aucun pack récent"); }
                            for path in self.recent.paths.clone() {
                                let name = path.file_name().unwrap_or_default().to_string_lossy();
                                if ui.button(name.as_ref()).on_hover_text(path.display().to_string()).clicked() {
                                    ui.close();
                                    self.open_selection(&path);
                                }
                            }
                        });
                    });
                    ui.add_enabled_ui(!self.dirty(), |ui| {
                        ui.menu_button("Importer", |ui| {
                            let choices = [
                                ("Fichier (.dynx)", "Configuration DynamX", vec!["dynx"]),
                                ("Pack (.dnxpack)", "Pack DynamX", vec!["dnxpack"]),
                                ("ZIP (.zip)", "Archive ZIP", vec!["zip"]),
                                ("RAR (.rar)", "Archive RAR", vec!["rar"]),
                            ];
                            if ui.button("Dossier de pack").clicked() {
                                ui.close();
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Choisir le dossier du pack")
                                    .pick_folder()
                                {
                                    self.open_selection(&path);
                                }
                            }
                            for (label, title, extensions) in choices {
                                if ui.button(label).clicked() {
                                    ui.close();
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_title(title)
                                        .add_filter(title, &extensions)
                                        .add_filter("Tous les fichiers", &["*"])
                                        .pick_file()
                                    {
                                        self.open_selection(&path);
                                    }
                                }
                            }
                        });
                    });
                    if ui
                        .add_enabled(
                            self.pack.is_some() || self.editor.is_some(),
                            egui::Button::new("Exporter"),
                        )
                        .clicked()
                    {
                        ui.close();
                        if self.pack.is_some() {
                            self.prepare_export();
                        } else {
                            self.export_current_file();
                        }
                    }
                    if ui
                        .add_enabled(
                            self.editor.as_ref().is_some_and(Editor::is_dirty),
                            egui::Button::new("Annuler les modifications du fichier"),
                        )
                        .clicked()
                    {
                        if let Some(editor) = &mut self.editor {
                            editor.revert();
                        }
                        self.error = None;
                        self.status = "Modifications annulées".into();
                        ui.close();
                    }
                    if ui.add_enabled(self.pack.as_ref().is_some_and(Pack::has_structure_changes),egui::Button::new("Annuler les ajouts et suppressions")).on_hover_text("Rétablit la liste d’origine et retire aussi les modèles et textures importés ; les modifications des configurations existantes sont conservées").clicked(){
                        if let Err(error)=self.undo_structure(){self.error=Some(error);}ui.close();
                    }
                    if (self.draft_pack.is_some() || self.draft_file) && ui.button("Abandonner le brouillon").on_hover_text("Ferme le nouveau pack ou fichier et abandonne ses changements en mémoire").clicked(){self.discard_draft();ui.close();}
                    if ui
                        .add_enabled(
                            !self.updater.checking(),
                            egui::Button::new("Vérifier les mises à jour"),
                        )
                        .clicked()
                    {
                        self.updater.check();
                        ui.close();
                    }
                });
                ui.menu_button("Outils",|ui|{
                    if ui.button("OpenMPS…").clicked(){self.openmps.begin(None);ui.close();}

                    if ui.button("Recherche").clicked(){self.search.open=true;ui.close();}
                    if ui.add_enabled(self.pack.is_some(),egui::Button::new("Modèles")).clicked(){self.model_browser=self.pack.as_ref().map(|p|model_browser::Browser::new(p.clone(),self.documents()));ui.close();}
                    if ui.add_enabled(self.pack.is_some(),egui::Button::new("Diagnostic")).clicked(){self.report=self.pack.as_ref().map(|p|report::Report::new(p.clone(),self.documents(),self.catalog.clone(),false));ui.close();}
                });
                ui.menu_button("Édition", |ui| {
                    if let Some(editor)=&mut self.editor {
                        if ui.add_enabled(editor.can_undo(),egui::Button::new("Annuler · Ctrl+Z")).clicked(){editor.undo();ui.close();}
                        if ui.add_enabled(editor.can_redo(),egui::Button::new("Rétablir · Ctrl+Y")).clicked(){editor.redo();ui.close();}
                    }
                });
                if ui.button("Propriétés").clicked() {
                    match Self::properties_directory() {
                        Ok(directory) => self.properties_editor = Some(properties_editor::PropertiesEditor::new(directory, &self.catalog)),
                        Err(error) => self.error = Some(error),
                    }
                }
                ui.hyperlink_to("Wiki", "https://wiki.dynamx.fr/fr/")
                    .on_hover_text("Documentation DynamX : syntaxe et paramètres des packs");
                ui.hyperlink_to("Aide CPM", "https://github.com/Florent-Marc/cpm/wiki")
                    .on_hover_text("Guides CPM : installation, création de packs et aperçu 3D");
                if self.dirty() {
                    ui.label("● Modifications non enregistrées");
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("Report bug", bug_report_url());
                    if let Some(release) = self.updater.available.clone() {
                        if release.can_install() {
                            let can_update = !self.dirty() && !self.editor.as_ref().is_some_and(|editor| editor.custom_parameter.is_some());
                            if ui.add_enabled(can_update, egui::Button::new(format!("Mettre à jour v{}", release.version)))
                                .on_hover_text("Enregistrer les modifications avant la mise à jour ; CPM redémarrera après installation.").clicked() {
                                self.updater.start_download();
                                ctx.request_repaint();
                            }
                        } else {
                            ui.hyperlink_to(format!("Mise à jour v{} (GitHub)", release.version), release.page_url);
                        }
                    }
                });
            });
        });
        egui::Panel::bottom("status").show(ui, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            }
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(format!("CPM v{}", env!("CARGO_PKG_VERSION"))).on_hover_text(&self.updater.message);
                });
            });
            if updating { ui.horizontal(|ui| { ui.spinner(); ui.label("Téléchargement et vérification de la mise à jour…"); }); }
            ui.small("Navigation dans le pack : les modifications restent en mémoire. File → Exporter pour sauvegarder le pack complet. Aperçu OBJ disponible à côté du nom de la configuration.");
        });
        egui::Panel::left("sections")
            .default_size(220.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Configurations");
                    if ui
                        .add_enabled(self.pack.is_some(), egui::Button::new("✎"))
                        .on_hover_text("Ajouter ou supprimer des configurations du pack")
                        .clicked()
                    {
                        self.manager = Some(creation::Manager::default());
                    }
                });
                let mut requested = None;
                egui::ScrollArea::vertical()
                    .id_salt("configs")
                    .max_height(250.0)
                    .show(ui, |ui| {
                        if let Some(path) = &self.standalone_path {
                            let issues =
                                self.editor.as_ref().map(Editor::issues).unwrap_or_default();
                            let mut text = egui::RichText::new(navigation::config_label(path));
                            if !issues.is_empty() {
                                text = text.color(egui::Color32::LIGHT_RED);
                            }
                            ui.label(text).on_hover_text(format!(
                                "{}\n{}",
                                path.display(),
                                issues.join("\n")
                            ));
                        }
                        if let Some(pack) = &self.pack {
                            for (category, indices) in navigation::config_groups(pack.configs()) {
                                let selected = self
                                    .selected_config
                                    .is_some_and(|index| indices.contains(&index));
                                let invalid = indices
                                    .iter()
                                    .any(|index| !self.issues_for(*index).is_empty());
                                let mut category_label =
                                    egui::RichText::new(format!("{category} ({})", indices.len()));
                                if invalid {
                                    category_label = category_label.color(egui::Color32::LIGHT_RED);
                                }
                                egui::CollapsingHeader::new(category_label)
                                    .id_salt(("config-category", &self.open_path, category))
                                    .default_open(selected)
                                    .show(ui, |ui| {
                                        for index in indices {
                                            let path = &pack.configs()[index];
                                            let dirty = if self.selected_config == Some(index) {
                                                self.editor.as_ref().is_some_and(Editor::is_dirty)
                                            } else {
                                                self.cached
                                                    .get(&index)
                                                    .is_some_and(Editor::is_dirty)
                                            };
                                            let label = format!(
                                                "{}{}",
                                                if dirty { "● " } else { "" },
                                                navigation::config_label(path)
                                            );
                                            let issues = self.issues_for(index);
                                            let mut text = egui::RichText::new(label);
                                            if !issues.is_empty() {
                                                text = text.color(egui::Color32::LIGHT_RED);
                                            }
                                            if ui
                                                .selectable_label(
                                                    self.selected_config == Some(index),
                                                    text,
                                                )
                                                .on_hover_text(format!(
                                                    "{}\n{}",
                                                    path.display(),
                                                    issues.join("\n")
                                                ))
                                                .clicked()
                                            {
                                                requested = Some(index);
                                            }
                                        }
                                    });
                            }
                        }
                    });
                if let Some(index) = requested {
                    self.switch_config(index);
                }
                ui.separator();
                ui.heading("Sections");
                egui::ScrollArea::vertical()
                    .id_salt("sections")
                    .show(ui, |ui| {
                        if ui
                            .selectable_label(self.selected_section.is_none(), "Racine")
                            .clicked()
                        {
                            self.selected_section = None;
                            self.pending_scroll = Some(None);
                        }
                        if let Some(editor) = &self.editor {
                            for (group, indices) in
                                navigation::section_groups(editor.document.sections())
                            {
                                let target = self
                                    .pending_scroll
                                    .flatten()
                                    .is_some_and(|index| indices.contains(&index));
                                let grouped = indices.len() > 1
                                    || editor.document.sections()[indices[0]].name != group;
                                let mut links = |ui: &mut egui::Ui| {
                                    for &index in &indices {
                                        let section = &editor.document.sections()[index];
                                        if ui
                                            .selectable_label(
                                                self.selected_section == Some(index),
                                                &section.name,
                                            )
                                            .on_hover_text(format!("Ligne {}", section.line))
                                            .clicked()
                                        {
                                            self.selected_section = Some(index);
                                            self.pending_scroll = Some(Some(index));
                                        }
                                    }
                                };
                                if grouped {
                                    egui::CollapsingHeader::new(format!(
                                        "{group} ({})",
                                        indices.len()
                                    ))
                                    .id_salt((
                                        "nav-section-group",
                                        &self.open_path,
                                        self.selected_config,
                                        &group,
                                    ))
                                    .open(if target { Some(true) } else { None })
                                    .show(ui, &mut links);
                                } else {
                                    links(ui);
                                }
                            }
                        }
                    });
            });
        let mut requested_preview = None;
        egui::CentralPanel::default().show(ui, |ui| {
            let Some(editor) = &mut self.editor else {
                ui.heading("Ouvrir un pack DynamX");
                ui.label("Utilise File → Importer, puis choisis un fichier, un dossier de pack, un pack .dnxpack ou une archive ZIP / RAR.");
                ui.label("Les commentaires et les champs d'addons sont conservés. Utilise File → Exporter pour enregistrer tes modifications.");
                return;
            };
            ui.horizontal_wrapped(|ui| {
                let path = if let (Some(pack),Some(index))=(&self.pack,self.selected_config) { pack.configs()[index].clone() } else { editor.path.clone() };
                ui.heading(config_name(&path)).on_hover_text(path.display().to_string());
                let request=cpm_core::ObjRequest::resolve(self.pack.as_ref(),&path,&editor.document);
                let hint=if self.gl.is_none(){"Le moteur OpenGL de l’aperçu n’est pas disponible".to_owned()}else{request.as_ref().map(|r|format!("Ouvrir {}",r.path.display())).unwrap_or_else(|e|e.clone())};
                if ui.add_enabled(request.is_ok() && self.gl.is_some(),egui::Button::new("Aperçu 3D")).on_hover_text(hint).clicked() {
                    if let Ok(request)=request { requested_preview=Some((request,path.clone())); }
                }
            });
            ui.label("Propriétés");
            let issues = editor.issues();
            if !issues.is_empty() {
                ui.colored_label(egui::Color32::LIGHT_RED, format!("{} erreur(s) de paramètres", issues.len()))
                    .on_hover_text(issues.join("\n"));
            }
            if editor.is_pack_info { ui.label(format!("Pack : {}", cpm_core::PackMetadata::from_document(&editor.document).name)); }
            let warnings = editor.warnings();
            if !warnings.is_empty() { ui.colored_label(egui::Color32::YELLOW, format!("{} avertissement(s)", warnings.len())).on_hover_text(warnings.join("\n")); }
            let document_id = (&self.open_path, self.selected_config);
            let jump = self.pending_scroll.take();
            ui.push_id(document_id, |ui| {
                egui::ScrollArea::vertical().id_salt("document-properties").show(ui, |ui| {
                    show_document(ui, editor, &mut self.error, jump);
                });
            });
        });
        if let Some((request, path)) = requested_preview {
            self.preview_config = path;
            self.preview_source = self.open_path.clone();
            self.preview_signature = self.scene_revision();
            self.preview_due = None;
            self.preview = Some(preview::Preview::open(
                request.with_documents(self.documents()),
                &ctx,
            ));
        }
        if let Some(editor) = &mut self.editor {
            custom_parameter_window(&ctx, editor, &mut self.error);
            editor.checkpoint();
        }
        if let Some(mut dialog) = self.export_dialog.take() {
            let mut keep = true;
            if let Some(result) = dialog.poll() {
                match result {
                    Ok(path) => {
                        self.adopt_export(&path);
                        keep = false;
                    }
                    Err(error) => dialog.error = Some(error),
                }
            }
            if keep {
                match dialog.show(&ctx) {
                    export_dialog::Action::Start => {
                        if let Some(pack) = &self.pack {
                            dialog.start(pack.clone(), self.documents(), self.catalog.clone());
                            ctx.request_repaint_after(std::time::Duration::from_millis(100));
                        }
                    }
                    export_dialog::Action::Close => keep = false,
                    export_dialog::Action::None => {}
                }
            }
            if keep {
                self.export_dialog = Some(dialog);
            }
        }
        if let Some(mut window) = self.properties_editor.take() {
            match window.show(&ctx) {
                properties_editor::Action::None => self.properties_editor = Some(window),
                properties_editor::Action::Close => {}
                properties_editor::Action::Applied(catalog, warnings) => {
                    self.apply_catalog(catalog);
                    self.status = "Propriétés enregistrées et appliquées".into();
                    self.error = (!warnings.is_empty()).then(|| warnings.join("\n"));
                }
            }
        }
        if let Some(mut creation) = self.creation.take() {
            match creation.show(
                &ctx,
                &self.catalog,
                self.pack.as_ref(),
                self.documents(),
                !self.dirty(),
            ) {
                creation::Action::None => self.creation = Some(creation),
                creation::Action::Close => {}
                creation::Action::Create(created) => {
                    if let Err(error) = self.create_document(created) {
                        creation.error = Some(error);
                        self.creation = Some(creation);
                    }
                }
            }
        }
        if let Some(mut manager) = self.manager.take() {
            let paths = self
                .pack
                .as_ref()
                .map(|p| p.configs().to_vec())
                .unwrap_or_default();
            match manager.show(&ctx, &paths) {
                creation::ManageAction::None => self.manager = Some(manager),
                creation::ManageAction::Close => {}
                creation::ManageAction::Add => self.creation = Some(creation::Creation::new(true)),
                creation::ManageAction::Delete(path) => {
                    manager.error = self.delete_configuration(&path).err();
                    self.manager = Some(manager);
                }
                creation::ManageAction::Import(path) => {
                    manager.error = self.import_configuration(&path).err();
                    self.manager = Some(manager);
                }
            }
        }
        if !self.modal() {
            if let Some(hit) = self.search.show(&ctx, self.pack.as_ref(), self.documents()) {
                self.switch_config(hit.index);
                self.selected_section = hit.section;
                self.pending_scroll = Some(hit.section);
            }
        }
        if let Some(mut report) = self.report.take() {
            match report.show(&ctx) {
                report::Action::None => self.report = Some(report),
                report::Action::Close => {}
                report::Action::Export => self.prepare_export_options(),
                report::Action::Jump(issue) => {
                    if let Some(index) = self
                        .pack
                        .as_ref()
                        .and_then(|p| p.configs().iter().position(|p| *p == issue.path))
                    {
                        self.switch_config(index);
                        self.selected_section = issue.section;
                        self.pending_scroll = Some(issue.section);
                        if let Some(editor) = &mut self.editor {
                            editor.focus_key = Some(issue.parameter);
                        }
                    }
                }
            }
        }
        if self
            .model_browser
            .as_ref()
            .is_some_and(|b| !b.matches(self.pack.as_ref()))
        {
            self.model_browser = None;
        }
        if !self.modal() {
            if let Some(mut browser) = self.model_browser.take() {
                let (open, selected) = browser.show(&ctx);
                if let Some(selected) = selected {
                    let request = match selected {
                        model_browser::Selection::Config(index) => {
                            self.switch_config(index);
                            self.preview_config = self
                                .editor
                                .as_ref()
                                .map(|e| e.path.clone())
                                .unwrap_or_default();
                            self.request_for(&self.preview_config)
                        }
                        model_browser::Selection::Asset(path) => {
                            self.preview_config = std::path::PathBuf::new();
                            cpm_core::Document::parse(&format!("Model: {}\n", path.display()))
                                .map_err(|e| e.to_string())
                                .and_then(|d| {
                                    cpm_core::ObjRequest::resolve(
                                        self.pack.as_ref(),
                                        std::path::Path::new("raw_model.dynx"),
                                        &d,
                                    )
                                })
                        }
                    };
                    match request {
                        Ok(request) => {
                            let mut preview = preview::Preview::open(
                                request.with_documents(self.documents()),
                                &ctx,
                            );
                            preview.live = !self.preview_config.as_os_str().is_empty();
                            self.preview = Some(preview);
                            self.preview_source = self.open_path.clone();
                            self.preview_signature = self.scene_revision();
                            self.preview_due = None;
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
                if open {
                    self.model_browser = Some(browser);
                }
            }
        }
        if self.preview_source != self.open_path {
            self.preview = None;
        }
        let revision = self.scene_revision();
        if let Some(preview) = &self.preview {
            if preview.live && revision != self.preview_signature {
                self.preview_signature = revision;
                self.preview_due = Some(std::time::Instant::now());
            }
        }
        if self.preview.as_ref().is_some_and(|p| p.reload_requested) {
            self.preview_due =
                Some(std::time::Instant::now() - std::time::Duration::from_millis(350));
        }
        if self.preview_due.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if self
            .preview
            .as_ref()
            .is_some_and(|p| (p.live || p.reload_requested) && !p.busy())
            && self
                .preview_due
                .is_some_and(|at| at.elapsed() >= std::time::Duration::from_millis(350))
        {
            self.preview_due = None;
            let request = self.request_for(&self.preview_config).map(|r| {
                r.with_documents(self.documents())
                    .with_variant(self.preview.as_ref().map(|p| p.variant).unwrap_or(0))
            });
            if let Some(preview) = &mut self.preview {
                match request {
                    Ok(request) => preview.refresh(request, &ctx),
                    Err(error) => preview.fail(error),
                }
            }
        }
        if let Some(mut preview) = self.preview.take() {
            preview.interactive = !self.modal() && !self.preview_config.as_os_str().is_empty();
            if self
                .editor
                .as_ref()
                .is_some_and(|e| e.path == self.preview_config)
            {
                preview.selected = self.selected_section;
            }
            if preview.show(&ctx, self.gl.as_ref()) {
                for edit in preview.take_edits() {
                    if self.report.is_some()
                        || self.export_dialog.is_some()
                        || self.properties_editor.is_some()
                    {
                        self.error =
                            Some("Terminer la fenêtre active avant de modifier l’aperçu".into());
                        continue;
                    }
                    if let Some(index) = self
                        .pack
                        .as_ref()
                        .and_then(|p| p.configs().iter().position(|p| *p == self.preview_config))
                    {
                        self.switch_config(index);
                    }
                    if let Some(editor) = self
                        .editor
                        .as_mut()
                        .filter(|e| e.path == self.preview_config)
                    {
                        match editor.apply_gizmo(&edit) {
                            Ok(()) => {
                                preview.reload_requested = true;
                                self.status = "Placement 3D appliqué · Ctrl+Z pour annuler".into();
                            }
                            Err(error) => self.error = Some(error),
                        }
                    }
                }
                if let Some(section) = preview.take_selection().filter(|_| preview.interactive) {
                    if let Some(index) = self
                        .pack
                        .as_ref()
                        .and_then(|p| p.configs().iter().position(|p| *p == self.preview_config))
                    {
                        self.switch_config(index);
                    }
                    self.selected_section = section;
                    self.pending_scroll = Some(section);
                }
                self.preview = Some(preview);
            }
        }
        if self.openmps.open || self.openmps.busy() {
            self.openmps.show(
                &ctx,
                self.pack.as_ref(),
                self.documents(),
                self.catalog.clone(),
                self.scene_revision(),
            );
        }
        if self.confirm_close {
            egui::Window::new("Modifications non enregistrées")
                .collapsible(false)
                .resizable(false)
                .show(&ctx, |ui| {
                    ui.label("Quitter et abandonner les modifications ?");
                    ui.horizontal(|ui| {
                        if ui.button("Revenir à l'éditeur").clicked() {
                            self.confirm_close = false;
                        }
                        if ui.button("Quitter sans enregistrer").clicked() {
                            self.allow_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
        }
    }
}

fn bug_report_url() -> String {
    format!("https://github.com/Florent-Marc/cpm/issues/new?title=Bug%20CPM&body=CPM%20v{}%0ASysteme%3A%20{}%20{}%0A%0ADescription%20du%20probleme%3A%0A%0AEtapes%20pour%20reproduire%3A%0A%0AResultat%20attendu%3A%0A", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH)
}

fn custom_parameter_window(
    ctx: &egui::Context,
    editor: &mut Editor,
    error: &mut Option<String>,
) -> [Option<egui::Rect>; 3] {
    let mut controls = [None; 3];
    let Some(section) = editor.custom_parameter else {
        return controls;
    };
    let mut open = true;
    let mut add = false;
    let mut cancel = false;
    egui::Window::new("Ajouter un paramètre d'addon")
        .open(&mut open)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label("Nom du paramètre");
            controls[0] = Some(ui.text_edit_singleline(&mut editor.custom_key).rect);
            ui.label("Valeur");
            controls[1] = Some(ui.text_edit_singleline(&mut editor.custom_value).rect);
            ui.horizontal(|ui| {
                let response =
                    ui.add_enabled(!editor.custom_key.is_empty(), egui::Button::new("Ajouter"));
                controls[2] = Some(response.rect);
                if response.clicked() {
                    add = true;
                }
                if ui.button("Annuler").clicked() {
                    cancel = true;
                }
            });
        });
    if add {
        match editor.add_parameter(
            section,
            &editor.custom_key.clone(),
            &editor.custom_value.clone(),
        ) {
            Ok(()) => {
                editor.custom_parameter = None;
                editor.custom_key.clear();
                editor.custom_value.clear();
                *error = None;
            }
            Err(message) => *error = Some(message),
        }
        ctx.request_repaint();
    }
    if !open || cancel {
        editor.custom_parameter = None;
    }
    controls
}

fn config_name(path: &std::path::Path) -> std::borrow::Cow<'_, str> {
    path.file_name().unwrap_or_default().to_string_lossy()
}

/// Toutes les propriétés restent dans la même vue ; la navigation ne filtre
/// plus le document, elle ouvre et rejoint simplement le bloc demandé.
fn show_document(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    error: &mut Option<String>,
    jump: Option<Option<usize>>,
) -> Vec<egui::Id> {
    if jump.is_some() {
        ui.style_mut().animation_time = 0.0;
    }
    let properties = editor.document.properties().to_vec();
    let sections = editor.document.sections().to_vec();
    let mut headers = Vec::with_capacity(sections.len());
    let diagnostics = editor.catalog.diagnostics(&editor.path, &editor.document);
    let root_problems: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.section.is_none())
        .cloned()
        .collect();
    let root = ui.label(diagnostic_text("Racine", &root_problems).strong());
    if jump == Some(None) {
        root.scroll_to_me(Some(egui::Align::Min));
    }
    show_properties(ui, editor, &properties, None, error);
    headers.resize(sections.len(), egui::Id::NULL);
    for (group, indices) in navigation::section_groups(&sections) {
        ui.separator();
        let grouped = indices.len() > 1 || sections[indices[0]].name != group;
        let target = jump.flatten().is_some_and(|index| indices.contains(&index));
        let mut children = |ui: &mut egui::Ui| {
            for &index in &indices {
                let section = &sections[index];
                let problems: Vec<_> = diagnostics
                    .iter()
                    .filter(|d| d.section == Some(index))
                    .cloned()
                    .collect();
                let response =
                    egui::CollapsingHeader::new(diagnostic_text(&section.name, &problems))
                        .id_salt(("config-section", index))
                        .open(if jump == Some(Some(index)) {
                            Some(true)
                        } else {
                            None
                        })
                        .show(ui, |ui| {
                            show_properties(ui, editor, &properties, Some(index), error);
                        });
                headers[index] = response.header_response.id;
                if jump == Some(Some(index)) {
                    response
                        .header_response
                        .scroll_to_me(Some(egui::Align::Min));
                }
            }
        };
        if grouped {
            let problems: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.section.is_some_and(|id| indices.contains(&id)))
                .cloned()
                .collect();
            egui::CollapsingHeader::new(diagnostic_text(
                format!("{group} ({})", indices.len()),
                &problems,
            ))
            .id_salt(("section-group", &group))
            .open(if target { Some(true) } else { None })
            .show(ui, &mut children);
        } else {
            children(ui);
        }
    }
    ui.separator();
    ui.collapsing("Texte du fichier après modification", |ui| {
        ui.monospace(editor.document.source());
    });
    if let Some(index) = editor.pending_delete.take() {
        match editor.remove_parameter(index) {
            Ok(()) => {
                *error = None;
                ui.ctx().request_repaint();
            }
            Err(message) => *error = Some(message),
        }
    }
    if let Some((section, key, value)) = editor.pending_add.take() {
        match editor.add_parameter(section, &key, &value) {
            Ok(()) => {
                *error = None;
                editor.custom_key.clear();
                ui.ctx().request_repaint();
            }
            Err(message) => *error = Some(message),
        }
    }
    headers
}

fn diagnostic_text(
    text: impl Into<String>,
    diagnostics: &[cpm_core::Diagnostic],
) -> egui::RichText {
    let label = egui::RichText::new(text.into());
    if diagnostics
        .iter()
        .any(|d| d.severity == cpm_core::Severity::Error)
    {
        label.color(egui::Color32::LIGHT_RED)
    } else if diagnostics
        .iter()
        .any(|d| d.severity == cpm_core::Severity::Warning)
    {
        label.color(egui::Color32::YELLOW)
    } else {
        label
    }
}

fn show_properties(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    properties: &[cpm_core::Property],
    section: Option<usize>,
    error: &mut Option<String>,
) {
    let definitions = editor
        .catalog
        .parameters(&editor.path, &editor.document, section);
    let diagnostics: Vec<_> = editor
        .catalog
        .diagnostics(&editor.path, &editor.document)
        .into_iter()
        .filter(|d| d.section == section)
        .collect();
    ui.push_id(("add-parameter", section), |ui| {
        ui.menu_button("+", |ui| {
            ui.label("Ajouter un paramètre");
            let mut specs = editor
                .catalog
                .parameters(&editor.path, &editor.document, section);
            specs.sort_by_key(|spec| !spec.required);
            let mut available = false;
            for spec in specs {
                if spec.is_present(&editor.document, section) {
                    continue;
                }
                available = true;
                let label = if spec.required {
                    format!("{} * obligatoire", spec.name)
                } else {
                    spec.name.to_owned()
                };
                let problems: Vec<_> = diagnostics
                    .iter()
                    .filter(|d| d.parameter == spec.name)
                    .cloned()
                    .collect();
                if ui
                    .button(diagnostic_text(label, &problems))
                    .on_hover_text(format!(
                        "{}\n{}",
                        spec.description,
                        problems
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join("\n")
                    ))
                    .clicked()
                {
                    editor.pending_add = Some((
                        section,
                        spec.name.to_owned(),
                        spec.default_value.unwrap_or_default(),
                    ));
                    ui.close();
                }
            }
            if !available {
                ui.label("Aucun paramètre connu manquant.");
            }
            ui.separator();
            if ui.button("Paramètre personnalisé (addon)…").clicked() {
                editor.custom_parameter = Some(section);
                ui.close();
            }
        })
        .response
        .on_hover_text("Ajouter un paramètre à cette section");
    });
    for diagnostic in &diagnostics {
        if let Some(definition) = definitions
            .iter()
            .find(|p| p.name == diagnostic.parameter && !p.is_present(&editor.document, section))
        {
            ui.horizontal(|ui| {
                ui.label(diagnostic_text(
                    format!("{} — absent", definition.name),
                    std::slice::from_ref(diagnostic),
                ))
                .on_hover_text(diagnostic.to_string());
                if ui.small_button("+ Ajouter").clicked() {
                    editor.pending_add = Some((
                        section,
                        definition.name.clone(),
                        definition.default_value.clone().unwrap_or_default(),
                    ));
                }
            });
        }
    }
    for (index, property) in properties
        .iter()
        .enumerate()
        .filter(|(_, p)| p.section == section)
    {
        ui.push_id(("property", index), |ui| {
            ui.horizontal(|ui| {
                if ui
                    .small_button("−")
                    .on_hover_text("Supprimer ce paramètre")
                    .clicked()
                {
                    editor.pending_delete = Some(index);
                }
                let definition = definitions
                    .iter()
                    .find(|p| p.name == property.key || p.aliases.contains(&property.key));
                let problems: Vec<_> = diagnostics
                    .iter()
                    .filter(|d| definition.is_some_and(|p| p.name == d.parameter))
                    .cloned()
                    .collect();
                let hint = definition
                    .map(|p| {
                        format!(
                            "{}\n\nLigne {} · {} · {} · {}\nDéfaut : {}",
                            p.description,
                            property.line,
                            p.variable_type.label(),
                            if p.required {
                                "obligatoire"
                            } else {
                                "facultatif"
                            },
                            p.severity.label(),
                            p.default_value.as_deref().unwrap_or("aucun")
                        )
                    })
                    .unwrap_or_else(|| format!("Ligne {} · paramètre personnalisé", property.line));
                let hint = if problems.is_empty() {
                    hint
                } else {
                    format!(
                        "{}\n{hint}",
                        problems
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                };
                ui.label(diagnostic_text(&property.key, &problems))
                    .on_hover_text(&hint);
                let color = if problems
                    .iter()
                    .any(|d| d.severity == cpm_core::Severity::Error)
                {
                    Some(egui::Color32::LIGHT_RED)
                } else if !problems.is_empty() {
                    Some(egui::Color32::YELLOW)
                } else {
                    None
                };
                let response =
                    parameter_input::input(ui, &mut editor.values[index], definition, color)
                        .on_hover_text(hint);
                if editor.focus_key.as_ref().is_some_and(|key| {
                    *key == property.key || definition.is_some_and(|d| d.name == *key)
                }) {
                    response.scroll_to_me(Some(egui::Align::Center));
                    editor.focus_key = None;
                }
                if response.changed() {
                    if let Err(edit_error) = editor.document.set_value(index, &editor.values[index])
                    {
                        *error = Some(edit_error.to_string());
                    }
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn importing_configuration_preserves_source_and_refuses_duplicate_names() {
        let source = tempfile::tempdir().unwrap();
        let pack = tempfile::tempdir().unwrap();
        let file = source.path().join("engine_imported.dynx");
        let text = "Power: 5000\nMaxRPM: 6000\nBraking: 100\n";
        std::fs::write(&file, text).unwrap();
        std::fs::write(
            pack.path().join("pack_info.dynx"),
            "PackName: Test\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        let mut app = CpmApp::default();
        app.open_selection(pack.path());
        app.import_configuration(&file).unwrap();
        assert!(app.dirty());
        assert_eq!(app.editor.as_ref().unwrap().document.source(), text);
        assert!(app.import_configuration(&file).is_err());
        assert_eq!(std::fs::read_to_string(file).unwrap(), text);
        assert!(!pack.path().join("engine_imported.dynx").exists());
    }

    #[test]
    fn deleting_and_undoing_structure_preserves_other_dirty_editors() {
        let temp = tempfile::tempdir().unwrap();
        for (name, text) in [
            ("pack_info.dynx", "PackName: Demo\nDcFileVersion: 12.5.0\n"),
            ("engine_a.dynx", "Power: 1000\nMaxRPM: 6000\nBraking: 100\n"),
            ("engine_b.dynx", "Power: 2000\nMaxRPM: 6000\nBraking: 100\n"),
        ] {
            std::fs::write(temp.path().join(name), text).unwrap();
        }
        let mut app = CpmApp::default();
        app.open_selection(temp.path());
        app.switch_config(2);
        app.editor
            .as_mut()
            .unwrap()
            .document
            .set_value(0, "8000")
            .unwrap();
        app.delete_configuration(std::path::Path::new("engine_a.dynx"))
            .unwrap();
        assert!(app.editor.as_ref().unwrap().is_dirty());
        assert_eq!(app.selected_config, Some(1));
        assert!(app.documents()[std::path::Path::new("engine_b.dynx")]
            .source()
            .contains("8000"));
        app.create_document(creation::Created::Configuration {
            path: "engine_c.dynx".into(),
            document: cpm_core::Document::parse("Power: 3000\nMaxRPM: 6000\nBraking: 100\n")
                .unwrap(),
        })
        .unwrap();
        assert!(app.dirty());
        app.undo_structure().unwrap();
        assert!(!app.pack.as_ref().unwrap().has_structure_changes());
        assert!(app.documents()[std::path::Path::new("engine_b.dynx")]
            .source()
            .contains("8000"));
        assert!(app.dirty());
        assert!(!temp.path().join("engine_c.dynx").exists());
    }
    #[test]
    fn new_pack_draft_exports_reopens_and_clears_dirty_state() {
        let document = cpm_core::Document::parse(
            "PackName: NewPack\nPackVersion: 1.0\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        let mut app = CpmApp::default();
        app.create_document(creation::Created::Pack {
            document,
            name: "NewPack".into(),
        })
        .unwrap();
        assert!(app.dirty());
        assert!(app.draft_pack.is_some());
        assert!(app.recent.paths.is_empty());
        app.prepare_export_options();
        assert_eq!(
            app.export_dialog.as_ref().unwrap().options.output_name,
            "NewPack"
        );
        let temp = tempfile::tempdir().unwrap();
        let mut options = app.export_dialog.as_ref().unwrap().options.clone();
        options.destination = temp.path().into();
        options.format = cpm_core::ExportFormat::Folder;
        let output = app
            .pack
            .as_ref()
            .unwrap()
            .export(&options, &app.documents())
            .unwrap();
        app.adopt_export(&output);
        assert!(!app.dirty());
        assert!(app.draft_pack.is_none());
        assert!(output.join("pack_info.dynx").is_file());
    }

    #[test]
    fn property_labels_show_errors_warnings_missing_and_clear_after_correction() {
        fn has_color(
            shapes: &[egui::epaint::ClippedShape],
            text: &str,
            color: egui::Color32,
        ) -> bool {
            fn matches(shape: &egui::epaint::Shape, text: &str, color: egui::Color32) -> bool {
                match shape {
                    egui::epaint::Shape::Text(shape) => {
                        shape.galley.job.text == text
                            && shape
                                .galley
                                .job
                                .sections
                                .iter()
                                .any(|s| s.format.color == color)
                    }
                    egui::epaint::Shape::Vec(shapes) => {
                        shapes.iter().any(|s| matches(s, text, color))
                    }
                    _ => false,
                }
            }
            shapes.iter().any(|s| matches(&s.shape, text, color))
        }
        let ctx = egui::Context::default();
        let mut editor = Editor::from_document(
            cpm_core::Document::parse("Power: wrong\nMaxRPM: 6000\n").unwrap(),
            std::path::Path::new("engine_demo.dynx"),
        );
        let mut error = None;
        let mut draw = |editor: &mut Editor| {
            let properties = editor.document.properties().to_vec();
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                show_properties(ui, editor, &properties, None, &mut error);
            });
            output.textures_delta.clear();
            output
        };
        let _ = draw(&mut editor);
        let output = draw(&mut editor);
        assert!(has_color(&output.shapes, "Power", egui::Color32::LIGHT_RED));
        assert!(has_color(
            &output.shapes,
            "Braking — absent",
            egui::Color32::LIGHT_RED
        ));
        let mut catalog = (*editor.catalog).clone();
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
            .severity = cpm_core::Severity::Warning;
        editor.catalog = std::sync::Arc::new(catalog);
        let output = draw(&mut editor);
        assert!(has_color(&output.shapes, "Power", egui::Color32::YELLOW));
        assert!(!has_color(
            &output.shapes,
            "Power",
            egui::Color32::LIGHT_RED
        ));
        editor.document.set_value(0, "1000").unwrap();
        editor.values[0] = "1000".into();
        let output = draw(&mut editor);
        assert!(!has_color(
            &output.shapes,
            "Power",
            egui::Color32::LIGHT_RED
        ));
        assert!(!has_color(&output.shapes, "Power", egui::Color32::YELLOW));
    }

    #[test]
    fn addon_window_stays_open_when_clicking_and_typing_then_adds_the_parameter() {
        let ctx = egui::Context::default();
        let mut editor = Editor::from_document(
            cpm_core::Document::parse("Name: Demo\n").unwrap(),
            std::path::Path::new("addon_test.dynx"),
        );
        editor.custom_parameter = Some(None);
        let mut error = None;
        let mut frame = |events: Vec<egui::Event>| {
            let mut controls = [None; 3];
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |_| {
                    controls = custom_parameter_window(&ctx, &mut editor, &mut error);
                },
            );
            output.textures_delta.clear();
            controls
        };
        frame(Vec::new());
        let controls = frame(Vec::new());
        let key = controls[0].unwrap().center();
        frame(vec![
            egui::Event::PointerMoved(key),
            egui::Event::PointerButton {
                pos: key,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        let controls = frame(vec![
            egui::Event::PointerButton {
                pos: key,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::Text("AddonFlag".into()),
        ]);
        let value = controls[1].unwrap().center();
        frame(vec![
            egui::Event::PointerMoved(value),
            egui::Event::PointerButton {
                pos: value,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        let controls = frame(vec![
            egui::Event::PointerButton {
                pos: value,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::Text("True".into()),
        ]);
        let add = controls[2].unwrap().center();
        frame(vec![
            egui::Event::PointerMoved(add),
            egui::Event::PointerButton {
                pos: add,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        frame(vec![egui::Event::PointerButton {
            pos: add,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert!(error.is_none());
        assert!(editor.custom_parameter.is_none());
        assert_eq!(editor.document.properties()[1].key, "AddonFlag");
        assert_eq!(editor.document.properties()[1].value, "True");
    }

    #[test]
    fn recent_history_tracks_only_successful_pack_opens() {
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("pack");
        std::fs::create_dir(&pack).unwrap();
        std::fs::write(
            pack.join("pack_info.dynx"),
            "PackName: Demo\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        let mut app = CpmApp::default();
        app.open_selection(&pack);
        assert_eq!(
            app.recent.paths,
            vec![std::fs::canonicalize(&pack).unwrap()]
        );
        app.open_selection(&pack.join("pack_info.dynx"));
        assert_eq!(app.recent.paths.len(), 1);
        app.open_selection(&temp.path().join("missing.zip"));
        assert_eq!(app.recent.paths.len(), 1);
        app.editor
            .as_mut()
            .unwrap()
            .document
            .set_value(0, "Changed")
            .unwrap();
        let other = temp.path().join("other");
        std::fs::create_dir(&other).unwrap();
        std::fs::write(other.join("pack_info.dynx"), "PackName: Other\n").unwrap();
        app.open_selection(&other);
        assert_eq!(app.recent.paths.len(), 1);
        assert!(app.error.as_ref().unwrap().contains("Enregistre"));
    }

    #[test]
    fn report_button_links_to_a_new_issue_with_version_context() {
        let url = bug_report_url();
        assert!(url.starts_with("https://github.com/Florent-Marc/cpm/issues/new?"));
        assert!(url.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn deleting_parameter_refreshes_values_and_required_field_status() {
        let ctx = egui::Context::default();
        let mut editor = Editor::from_document(
            cpm_core::Document::parse("PackName: Demo\nDcFileVersion: 12.5.0\n").unwrap(),
            std::path::Path::new("pack_info.dynx"),
        );
        editor.pending_delete = Some(1);
        let mut error = None;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            show_document(ui, &mut editor, &mut error, None);
        });
        output.textures_delta.clear();
        assert!(editor.is_dirty());
        assert_eq!(editor.values.len(), 1);
        assert!(editor.issues()[0].contains("DcFileVersion"));
        editor.revert();
        assert!(editor.issues().is_empty());
    }

    #[test]
    fn adding_missing_parameter_refreshes_navigation_and_export_state() {
        let root = std::env::temp_dir().join(format!("cpm-parameter-ui-{}", std::process::id()));
        let source = root.join("source");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("pack_info.dynx"), "PackName: Test\n").unwrap();
        let mut app = CpmApp::default();
        app.open_selection(&source);
        assert!(app
            .issues_for(0)
            .iter()
            .any(|issue| issue.contains("DcFileVersion")));
        app.editor.as_mut().unwrap().pending_add =
            Some((None, "DcFileVersion".into(), "12.5.0".into()));
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
        output.textures_delta.clear();
        assert!(app.issues_for(0).is_empty());
        let editor = app.editor.as_ref().unwrap();
        assert_eq!(editor.values.len(), editor.document.properties().len());
        assert_eq!(editor.values.last().unwrap(), "12.5.0");
        assert!(app.dirty());
        app.prepare_export_options();
        let mut options = app.export_dialog.as_ref().unwrap().options.clone();
        options.output_name = "corrected".into();
        assert!(app
            .pack
            .as_ref()
            .unwrap()
            .export(&options, &app.documents())
            .is_ok());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_dialog_uses_current_metadata_and_all_cached_configurations() {
        let temp = std::env::temp_dir().join(format!("cpm-export-ui-{}", std::process::id()));
        let source = temp.join("source");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(
            source.join("pack_info.dynx"),
            "PackName: Demo\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        std::fs::write(source.join("vehicle_car.dynx"), "Name: Original\nDescription: Test\nModel: obj/car.obj\nCenterOfGravityOffset: 0 0 0\nDragCoefficient: 0.3\nEmptyMass: 1500\n").unwrap();
        let mut app = CpmApp::default();
        app.open_selection(&source);
        app.editor
            .as_mut()
            .unwrap()
            .document
            .set_value(0, "EditedPack")
            .unwrap();
        app.switch_config(1);
        app.editor
            .as_mut()
            .unwrap()
            .document
            .set_value(0, "EditedCar")
            .unwrap();
        app.prepare_export_options();
        let dialog = app.export_dialog.as_mut().unwrap();
        assert_eq!(dialog.options.metadata.name, "EditedPack");
        dialog.options.output_name = "copy".into();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
        output.textures_delta.clear();
        let options = app.export_dialog.as_ref().unwrap().options.clone();
        let result = app
            .pack
            .as_ref()
            .unwrap()
            .export(&options, &app.documents())
            .unwrap();
        app.export_dialog = None;
        app.adopt_export(&result);
        assert!(!app.dirty());
        assert_eq!(
            app.editor.as_ref().unwrap().document.properties()[0].value,
            "EditedPack"
        );
        app.switch_config(1);
        assert_eq!(
            app.editor.as_ref().unwrap().document.properties()[0].value,
            "EditedCar"
        );
        assert!(std::fs::read_to_string(source.join("vehicle_car.dynx"))
            .unwrap()
            .contains("Original"));
        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn section_navigation_opens_target_and_keeps_independent_fold_states() {
        let ctx = egui::Context::default();
        let document = cpm_core::Document::parse(
            "Name: Demo\nSeat{\n Driver: True\n}\nSeat{\n Driver: False\n}\n",
        )
        .unwrap();
        let mut editor = Editor::from_document(document, std::path::Path::new("vehicle_demo.dynx"));
        let mut error = None;
        let mut states = Vec::new();
        for (jump, expected) in [
            (None, vec![false, false]),
            (Some(Some(1)), vec![false, true]),
            (Some(Some(0)), vec![true, true]),
            (Some(None), vec![true, true]),
        ] {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                let headers = show_document(ui, &mut editor, &mut error, jump);
                if jump.is_none() {
                    assert!(headers.iter().all(|id| *id == egui::Id::NULL));
                    states = vec![false, false];
                    return;
                }
                assert_ne!(headers[0], headers[1]);
                states = headers
                    .iter()
                    .map(|id| {
                        egui::collapsing_header::CollapsingState::load(ui.ctx(), *id)
                            .unwrap()
                            .is_open()
                    })
                    .collect();
            });
            output.textures_delta.clear();
            assert_eq!(states, expected);
        }
        assert!(error.is_none());
        assert_eq!(editor.document.properties()[1].value, "True");
        assert_eq!(editor.document.properties()[2].value, "False");
    }

    #[test]
    fn configuration_names_hide_folders_even_when_basenames_match() {
        assert_eq!(
            config_name(std::path::Path::new("vehicles/car/vehicle_test.dynx")),
            "vehicle_test.dynx"
        );
        assert_eq!(
            config_name(std::path::Path::new("other/vehicle_test.dynx")),
            "vehicle_test.dynx"
        );
    }

    #[test]
    fn selecting_files_or_packs_preserves_edits_and_keeps_state_on_failure() {
        let directory = std::env::temp_dir().join(format!("cpm-selection-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let config = directory.join("item_seul.DYNX");
        std::fs::write(&config, "Name: Standalone\n").unwrap();
        let mut app = CpmApp::default();
        app.open_selection(&config);
        assert!(app.error.is_none());
        assert!(app.pack.is_none());
        assert_eq!(app.standalone_path.as_ref(), Some(&config));
        app.editor
            .as_mut()
            .unwrap()
            .document
            .set_value(0, "Changed")
            .unwrap();
        std::fs::write(directory.join("pack_info.dynx"), "PackName: Pack\n").unwrap();
        app.open_selection(&directory);
        assert!(app.error.is_some());
        assert_eq!(
            app.editor.as_ref().unwrap().document.properties()[0].value,
            "Changed"
        );
        app.editor.as_mut().unwrap().revert();
        app.open_selection(&directory);
        assert!(app.error.is_none());
        assert!(app.pack.is_some());
        assert!(app.standalone_path.is_none());
        app.open_selection(&directory.join("missing.dnxpack"));
        assert!(app.error.is_some());
        assert!(app.pack.is_some());
        app.open_selection(&config);
        assert!(app.error.is_none());
        assert!(app.pack.is_none());
        assert!(app.cached.is_empty());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn renders_empty_and_loaded_editor_without_a_window() {
        let ctx = egui::Context::default();
        let mut app = CpmApp::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
        assert!(!output.shapes.is_empty());
        output.textures_delta.clear();
        let path = std::env::temp_dir().join(format!("cpm-ui-{}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(
            path.join("pack_info.dynx"),
            "PackName: Test\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        std::fs::write(
            path.join("vehicle_test.dynx"),
            "Name: Demo\nSeat_front{\n Driver: True\n}\n",
        )
        .unwrap();
        app.open_path = path.display().to_string();
        app.open();
        assert!(app.error.is_none());
        app.switch_config(1);
        for section in [None, Some(0)] {
            app.selected_section = section;
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
            assert!(!output.shapes.is_empty());
            output.textures_delta.clear();
        }
        app.editor
            .as_mut()
            .unwrap()
            .document
            .set_value(0, "Changed")
            .unwrap();
        app.confirm_close = true;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
        assert!(!output.shapes.is_empty());
        output.textures_delta.clear();
        app.switch_config(0);
        assert!(app.dirty());
        app.switch_config(1);
        assert_eq!(
            app.editor.as_ref().unwrap().document.properties()[0].value,
            "Changed"
        );
        std::fs::remove_dir_all(path).unwrap();
    }
}

#[cfg(test)]
mod tools_menu_tests {
    use super::*;
    fn text_center(shapes: &[egui::epaint::ClippedShape], label: &str) -> egui::Pos2 {
        fn find(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Pos2> {
            match shape {
                egui::epaint::Shape::Text(t) if t.galley.job.text == label => {
                    Some(t.pos + t.galley.rect.center().to_vec2())
                }
                egui::epaint::Shape::Vec(v) => v.iter().find_map(|s| find(s, label)),
                _ => None,
            }
        }
        shapes
            .iter()
            .find_map(|s| find(&s.shape, label))
            .unwrap_or_else(|| panic!("control not drawn: {label}"))
    }
    fn frame(
        app: &mut CpmApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> Vec<egui::epaint::ClippedShape> {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1400.0, 900.0),
                )),
                events,
                ..Default::default()
            },
            |ui| app.draw(ui),
        );
        output.textures_delta.clear();
        output.shapes
    }
    fn click(
        app: &mut CpmApp,
        ctx: &egui::Context,
        pos: egui::Pos2,
    ) -> Vec<egui::epaint::ClippedShape> {
        frame(app, ctx, vec![egui::Event::PointerMoved(pos)]);
        frame(
            app,
            ctx,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame(
            app,
            ctx,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(app, ctx, vec![])
    }
    #[test]
    fn tools_menu_opens_models_search_and_diagnostics() {
        for tool in ["Modèles", "Recherche", "Diagnostic", "OpenMPS…"] {
            let t = tempfile::tempdir().unwrap();
            std::fs::write(
                t.path().join("pack_info.dynx"),
                "PackName: Demo\nDcFileVersion: 12.5.0\n",
            )
            .unwrap();
            let mut app = CpmApp::default();
            app.open_selection(t.path());
            let ctx = egui::Context::default();
            frame(&mut app, &ctx, vec![]);
            let shapes = frame(&mut app, &ctx, vec![]);
            let tools = text_center(&shapes, "Outils");
            let shapes = click(&mut app, &ctx, tools);
            let control = text_center(&shapes, tool);
            click(&mut app, &ctx, control);
            match tool {
                "Modèles" => assert!(app.model_browser.is_some()),
                "Recherche" => assert!(app.search.open),
                "OpenMPS…" => assert!(app.openmps.open),
                _ => assert!(app.report.is_some()),
            }
        }
    }
    #[test]
    fn export_starts_with_report_and_modal_blocks_mutations() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(
            t.path().join("pack_info.dynx"),
            "PackName: Demo\nDcFileVersion: 12.5.0\n",
        )
        .unwrap();
        let mut app = CpmApp::default();
        app.open_selection(t.path());
        app.prepare_export();
        assert!(app.report.is_some());
        assert!(app.export_dialog.is_none());
        assert!(app.modal());
    }
}
