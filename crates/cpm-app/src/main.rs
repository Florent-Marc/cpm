mod editor;

use cpm_core::Pack;
use editor::Editor;
use eframe::egui;
use std::collections::BTreeMap;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 740.0])
            .with_min_inner_size([760.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "CPM — Éditeur de configuration",
        options,
        Box::new(|_| Ok(Box::new(CpmApp::default()))),
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
    save_path: String,
    selected_section: Option<usize>,
    status: String,
    error: Option<String>,
    confirm_close: bool,
    allow_close: bool,
}

impl CpmApp {
    fn dirty(&self) -> bool {
        self.editor.as_ref().is_some_and(Editor::is_dirty)
            || self.cached.values().any(Editor::is_dirty)
    }

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
                    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    self.save_path = path
                        .with_file_name(format!("{stem}_edited.dynx"))
                        .display()
                        .to_string();
                    self.editor = Some(editor);
                    self.pack = None;
                    self.cached.clear();
                    self.selected_config = None;
                    self.selected_section = None;
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
                self.switch_config(0);
            }
            Err(error) => self.error = Some(error),
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
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let parent = if pack.source().is_dir() {
            pack.source()
        } else {
            pack.source().parent().unwrap_or(std::path::Path::new("."))
        };
        self.save_path = parent
            .join(if pack.source().is_dir() {
                path.as_path()
            } else {
                std::path::Path::new(path.file_name().unwrap_or_default())
            })
            .with_file_name(format!("{stem}_edited.dynx"))
            .display()
            .to_string();
        if let (Some(previous), Some(editor)) = (self.selected_config, self.editor.take()) {
            self.cached.insert(previous, editor);
        }
        self.editor = Some(next);
        self.selected_config = Some(index);
        self.selected_section = None;
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
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty() && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("CPM");
                ui.label("Éditeur de configuration DynamX");
                if self.dirty() {
                    ui.label("● Modifications non enregistrées");
                }
            });
            ui.horizontal_wrapped(|ui| {
                let can_open = !self.dirty();
                if ui.add_enabled(can_open, egui::Button::new("Ouvrir un fichier…")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Ouvrir une configuration ou une archive de pack")
                        .add_filter("Configurations et packs DynamX", &["dynx", "dnxpack", "zip"])
                        .add_filter("Configurations", &["dynx"])
                        .add_filter("Archives de pack", &["dnxpack", "zip"])
                        .pick_file()
                    { self.open_selection(&path); }
                }
                if ui.add_enabled(can_open, egui::Button::new("Ouvrir un dossier de pack…")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Choisir le dossier contenant pack_info.dynx")
                        .pick_folder()
                    { self.open_selection(&path); }
                }
                if !can_open { ui.label("Enregistre ou annule les modifications avant de changer de pack ou fichier."); }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Chemin");
                ui.add(
                    egui::TextEdit::singleline(&mut self.open_path)
                        .desired_width(500.0)
                        .hint_text("Configuration .dynx, dossier ou archive .dnxpack"),
                );
                if ui
                    .add_enabled(
                        !self.dirty() && !self.open_path.is_empty(),
                        egui::Button::new("Ouvrir"),
                    )
                    .clicked()
                {
                    self.open();
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Nouvelle copie");
                ui.add(
                    egui::TextEdit::singleline(&mut self.save_path)
                        .desired_width(500.0)
                        .hint_text("Chemin de la copie à créer"),
                );
                if ui
                    .add_enabled(
                        self.editor.is_some() && !self.save_path.is_empty(),
                        egui::Button::new("Enregistrer une copie"),
                    )
                    .clicked()
                {
                    if let Some(editor) = &mut self.editor {
                        match editor.save_copy(std::path::Path::new(&self.save_path)) {
                            Ok(()) => {
                                self.status = format!("Copie enregistrée : {}", self.save_path);
                                self.error = None;
                            }
                            Err(error) => self.error = Some(error),
                        }
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
                }
            });
        });
        egui::Panel::bottom("status").show(ui, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            }
            ui.label(&self.status);
            ui.small("Navigation dans le pack : les modifications restent en mémoire. Enregistrement par copie de configuration ; export du pack complet et 3D à venir.");
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
                            ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                        }
                        if let Some(pack) = &self.pack {
                            for (index, path) in pack.configs().iter().enumerate() {
                                let dirty = if self.selected_config == Some(index) {
                                    self.editor.as_ref().is_some_and(Editor::is_dirty)
                                } else {
                                    self.cached.get(&index).is_some_and(Editor::is_dirty)
                                };
                                let label =
                                    format!("{}{}", if dirty { "● " } else { "" }, path.display());
                                if ui
                                    .selectable_label(self.selected_config == Some(index), label)
                                    .clicked()
                                {
                                    requested = Some(index);
                                }
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
                        ui.selectable_value(&mut self.selected_section, None, "Racine");
                        if let Some(editor) = &self.editor {
                            for (index, section) in editor.document.sections().iter().enumerate() {
                                ui.selectable_value(
                                    &mut self.selected_section,
                                    Some(index),
                                    format!("{} · ligne {}", section.name, section.line),
                                );
                            }
                        }
                    });
            });
        egui::CentralPanel::default().show(ui, |ui| {
            let Some(editor) = &mut self.editor else {
                ui.heading("Ouvrir un pack DynamX");
                ui.label("Clique sur Ouvrir un fichier pour une configuration ou une archive, ou sur Ouvrir un dossier de pack pour un pack non compressé.");
                ui.label("Les commentaires et les champs d'addons sont conservés. La sauvegarde crée une nouvelle copie sans écraser de fichier existant.");
                return;
            };
            if let (Some(pack), Some(index)) = (&self.pack, self.selected_config) {
                ui.heading(pack.configs()[index].display().to_string());
            }
            if let Some(path) = &self.standalone_path {
                ui.heading(path.file_name().unwrap_or_default().to_string_lossy());
            }
            ui.label("Propriétés");
            if editor.is_pack_info {
                match cpm_core::PackInfo::from_document(&editor.document) {
                    Ok(info) => { ui.label(format!("Pack : {}", info.pack_name)); }
                    Err(errors) => { for error in errors { ui.colored_label(egui::Color32::YELLOW, error.to_string()); } }
                }
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                let properties = editor.document.properties().to_vec();
                for (index, property) in properties.iter().enumerate().filter(|(_, p)| p.section == self.selected_section) {
                    ui.push_id(index, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(&property.key).on_hover_text(format!("Ligne {}", property.line));
                            if ui.add(egui::TextEdit::singleline(&mut editor.values[index]).desired_width(f32::INFINITY)).changed() {
                                if let Err(error) = editor.document.set_value(index, &editor.values[index]) { self.error = Some(error.to_string()); }
                            }
                        });
                    });
                }
                ui.separator();
                ui.collapsing("Texte du fichier après modification", |ui| { ui.monospace(editor.document.source()); });
            });
        });
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

#[cfg(test)]
mod tests {
    use super::*;

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
