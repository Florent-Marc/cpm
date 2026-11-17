mod editor;
mod export_dialog;
mod navigation;
mod update;

use cpm_core::Pack;
use editor::Editor;
use eframe::egui;
use std::collections::BTreeMap;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
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
        Box::new(|_| {
            let mut app = CpmApp::default();
            app.updater.check();
            Ok(Box::new(app))
        }),
    )
}

#[derive(Default)]
struct CpmApp {
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
    fn dirty(&self) -> bool {
        self.editor.as_ref().is_some_and(Editor::is_dirty)
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
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("dynx"))
        {
            match Editor::open(path) {
                Ok(editor) => {
                    self.editor = Some(editor);
                    self.pack = None;
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
                self.standalone_path = None;
                self.open_path = path.display().to_string();
                self.editor = None;
                self.cached.clear();
                self.selected_config = None;
                self.status = "Pack ouvert".into();
                self.start_validation();
                self.switch_config(0);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn start_validation(&mut self) {
        self.validation.clear();
        let Some(pack) = self.pack.clone() else {
            return;
        };
        let (sender, receiver) = std::sync::mpsc::channel();
        self.validation_worker = Some(receiver);
        std::thread::spawn(move || {
            for (index, path) in pack.configs().iter().enumerate() {
                let issues = match pack.read_config(index) {
                    Ok(document) => cpm_core::configuration_issues(path, &document),
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
        let filename = format!(
            "{}_edited.dynx",
            editor
                .path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        );
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Exporter la configuration")
            .set_file_name(filename)
            .add_filter("Configuration DynamX", &["dynx"])
            .save_file()
        {
            match editor.save_copy(&path) {
                Ok(()) => {
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
            let documents = self.documents();
            let metadata = documents
                .get(std::path::Path::new("pack_info.dynx"))
                .cloned()
                .map(Ok)
                .unwrap_or_else(|| pack.read_config(0));
            match metadata {
                Ok(document) => {
                    self.export_dialog = Some(export_dialog::ExportDialog::new(
                        pack,
                        cpm_core::PackMetadata::from_document(&document),
                    ))
                }
                Err(error) => self.error = Some(error),
            }
        }
    }

    fn adopt_export(&mut self, path: &std::path::Path) {
        match Pack::open(path) {
            Ok(pack) => {
                self.pack = Some(pack);
                self.editor = None;
                self.cached.clear();
                self.selected_config = None;
                self.selected_section = None;
                self.pending_scroll = None;
                self.open_path = path.display().to_string();
                self.standalone_path = None;
                self.start_validation();
                self.switch_config(0);
                self.status = format!("Pack exporté : {}", path.display());
            }
            Err(error) => {
                self.error = Some(format!(
                    "Export terminé, mais réouverture impossible : {error}"
                ))
            }
        }
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
        let next = if let Some(editor) = self.cached.remove(&index) {
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
        if let (Some(previous), Some(editor)) = (self.selected_config, self.editor.take()) {
            self.cached.insert(previous, editor);
        }
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
        if self.export_dialog.is_some() || updating || self.restarting {
            ui.disable();
        }
        egui::Panel::top("toolbar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.add_enabled_ui(!self.dirty(), |ui| {
                        ui.menu_button("Importer", |ui| {
                            let choices = [
                                ("Fichier (.dynx)…", "Configuration DynamX", vec!["dynx"]),
                                ("Pack (.dnxpack)…", "Pack DynamX", vec!["dnxpack"]),
                                ("ZIP (.zip)…", "Archive ZIP", vec!["zip"]),
                                ("RAR (.rar)…", "Archive RAR", vec!["rar"]),
                            ];
                            if ui.button("Dossier de pack…").clicked() {
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
                            egui::Button::new("Exporter…"),
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
                ui.hyperlink_to("Wiki", "https://wiki.dynamx.fr/fr/");
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
            ui.small("Navigation dans le pack : les modifications restent en mémoire. File → Exporter pour sauvegarder le pack complet. Prévisualisation 3D à venir.");
        });
        egui::Panel::left("sections")
            .default_size(220.0)
            .show(ui, |ui| {
                ui.heading("Configurations");
                let mut requested = None;
                egui::ScrollArea::vertical()
                    .id_salt("configs")
                    .max_height(250.0)
                    .show(ui, |ui| {
                        if let Some(path) = &self.standalone_path {
                            let issues =
                                self.editor.as_ref().map(Editor::issues).unwrap_or_default();
                            let mut text = egui::RichText::new(config_name(path));
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
                                                config_name(path)
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
        egui::CentralPanel::default().show(ui, |ui| {
            let Some(editor) = &mut self.editor else {
                ui.heading("Ouvrir un pack DynamX");
                ui.label("Utilise File → Importer, puis choisis un fichier, un dossier de pack, un pack .dnxpack ou une archive ZIP / RAR.");
                ui.label("Les commentaires et les champs d'addons sont conservés. Utilise File → Exporter pour enregistrer tes modifications.");
                return;
            };
            if let (Some(pack), Some(index)) = (&self.pack, self.selected_config) {
                ui.heading(config_name(&pack.configs()[index])).on_hover_text(pack.configs()[index].display().to_string());
            }
            if let Some(path) = &self.standalone_path {
                ui.heading(path.file_name().unwrap_or_default().to_string_lossy());
            }
            ui.label("Propriétés");
            let issues = editor.issues();
            if !issues.is_empty() {
                ui.colored_label(egui::Color32::LIGHT_RED, format!("{} problème(s) de paramètres obligatoires", issues.len()))
                    .on_hover_text(issues.join("\n"));
            }
            if editor.is_pack_info {
                match cpm_core::PackInfo::from_document(&editor.document) {
                    Ok(info) => { ui.label(format!("Pack : {}", info.pack_name)); }
                    Err(errors) => { for error in errors { ui.colored_label(egui::Color32::YELLOW, error.to_string()); } }
                }
            }
            let document_id = (&self.open_path, self.selected_config);
            let jump = self.pending_scroll.take();
            ui.push_id(document_id, |ui| {
                egui::ScrollArea::vertical().id_salt("document-properties").show(ui, |ui| {
                    show_document(ui, editor, &mut self.error, jump);
                });
            });
        });
        if let Some(editor) = &mut self.editor {
            custom_parameter_window(&ctx, editor, &mut self.error);
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
                            dialog.start(pack.clone(), self.documents());
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
    let root = ui.strong("Racine");
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
                let response = egui::CollapsingHeader::new(&section.name)
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
            egui::CollapsingHeader::new(format!("{group} ({})", indices.len()))
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

fn show_properties(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    properties: &[cpm_core::Property],
    section: Option<usize>,
    error: &mut Option<String>,
) {
    ui.push_id(("add-parameter", section), |ui| {
        ui.menu_button("+", |ui| {
            ui.label("Ajouter un paramètre");
            let mut specs = cpm_core::parameter_catalog(&editor.path, &editor.document, section);
            specs.sort_by_key(|spec| !spec.required);
            let mut available = false;
            for spec in specs {
                if cpm_core::has_parameter(&editor.document, section, &spec) {
                    continue;
                }
                available = true;
                let label = if spec.required {
                    format!("{} * obligatoire", spec.key)
                } else {
                    spec.key.to_owned()
                };
                if ui.button(label).clicked() {
                    editor.pending_add =
                        Some((section, spec.key.to_owned(), spec.default_value.to_owned()));
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
                ui.label(&property.key)
                    .on_hover_text(format!("Ligne {}", property.line));
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut editor.values[index])
                            .desired_width(f32::INFINITY),
                    )
                    .changed()
                {
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
        app.prepare_export();
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
        app.prepare_export();
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
