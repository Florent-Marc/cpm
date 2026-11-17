mod editor;

use editor::Editor;
use eframe::egui;

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
    open_path: String,
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
    }

    fn open(&mut self) {
        match Editor::open(std::path::Path::new(&self.open_path)) {
            Ok(editor) => {
                let path = std::path::Path::new(&self.open_path);
                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                self.save_path = path
                    .with_file_name(format!("{stem}_edited.dynx"))
                    .display()
                    .to_string();
                self.editor = Some(editor);
                self.selected_section = None;
                self.status = "Fichier ouvert".into();
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
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
                ui.label("Fichier .dynx");
                ui.add(
                    egui::TextEdit::singleline(&mut self.open_path)
                        .desired_width(500.0)
                        .hint_text("Chemin du fichier à ouvrir"),
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
                    .add_enabled(self.dirty(), egui::Button::new("Annuler les modifications"))
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
            ui.small("Première version : un fichier à la fois. Dossiers, archives et 3D à venir.");
        });
        egui::Panel::left("sections")
            .default_size(220.0)
            .show(ui, |ui| {
                ui.heading("Sections");
                egui::ScrollArea::vertical().show(ui, |ui| {
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
                ui.heading("Ouvrir une configuration");
                ui.label("Indique le chemin d'un fichier .dynx dans la barre supérieure, puis clique sur Ouvrir.");
                ui.label("Les commentaires et les champs d'addons sont conservés. La sauvegarde crée une nouvelle copie sans écraser de fichier existant.");
                return;
            };
            ui.heading("Propriétés");
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
    fn renders_empty_and_loaded_editor_without_a_window() {
        let ctx = egui::Context::default();
        let mut app = CpmApp::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
        assert!(!output.shapes.is_empty());
        output.textures_delta.clear();
        let path = std::env::temp_dir().join(format!("cpm-ui-{}.dynx", std::process::id()));
        std::fs::write(&path, "Name: Demo\nSeat_front{\n Driver: True\n}\n").unwrap();
        app.open_path = path.display().to_string();
        app.open();
        assert!(app.error.is_none());
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
        std::fs::remove_file(path).unwrap();
    }
}
