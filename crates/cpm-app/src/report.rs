use cpm_core::{Document, Pack, PackIssue, ParameterCatalog, Severity};
use eframe::egui;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{mpsc, Arc},
};
pub enum Action {
    None,
    Close,
    Jump(PackIssue),
    Export { force: bool },
}
pub struct Report {
    issues: Vec<PackIssue>,
    receiver: Option<mpsc::Receiver<Vec<PackIssue>>>,
    export: bool,
    error: Option<String>,
}
impl Report {
    pub fn new(
        pack: Pack,
        documents: BTreeMap<PathBuf, Document>,
        catalog: Arc<ParameterCatalog>,
        export: bool,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(pack.audit(&documents, &catalog));
        });
        Self {
            issues: Vec::new(),
            receiver: Some(rx),
            export,
            error: None,
        }
    }
    pub fn show(&mut self, ctx: &egui::Context) -> Action {
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok(issues) => {
                    self.issues = issues;
                    self.receiver = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("Diagnostic interrompu : relancer le contrôle".into());
                    self.receiver = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
        }
        let mut action = Action::None;
        let mut open = true;
        egui::Window::new("Diagnostic du pack").open(&mut open).default_size([720.0,450.0]).collapsible(false).show(ctx,|ui| {
            let errors=self.issues.iter().filter(|i|i.severity==Severity::Error).count();
            ui.label(format!("{errors} erreur(s) · {} avertissement(s)",self.issues.len()-errors));
            ui.weak("Clique une ligne pour ouvrir la configuration et sa section. Relance le contrôle après correction.");
            if self.receiver.is_some(){ui.spinner();ui.label("Analyse des configurations et références…");}
            if let Some(error)=&self.error{ui.colored_label(crate::theme::ERROR,error);}
            egui::ScrollArea::vertical().max_height(340.0).show(ui,|ui| {
                for issue in &self.issues {
                    let color=if issue.severity==Severity::Error{crate::theme::ERROR}else{crate::theme::WARNING};
                    if ui.button(egui::RichText::new(format!("{} · {} : {}",issue.path.display(),issue.parameter,issue.message)).color(color)).clicked(){action=Action::Jump(issue.clone());}
                }
            });
            if self.export && errors > 0 { ui.colored_label(crate::theme::WARNING, "Forcer ignore les erreurs de configuration ; le pack exporté peut ne pas fonctionner en jeu."); }
            ui.horizontal(|ui| {
                let ready = self.receiver.is_none() && self.error.is_none();
                if self.export {
                    if ui.add_enabled(ready && !self.issues.is_empty(), egui::Button::new("Corriger")).clicked() {
                        action = self.issues.iter().find(|i| i.severity == Severity::Error).or(self.issues.first()).cloned().map(Action::Jump).unwrap_or(Action::Close);
                    }
                    let label = if errors > 0 { "Forcer l’export" } else { "Continuer vers l’export" };
                    if ui.add_enabled(ready, egui::Button::new(label)).clicked() {action = Action::Export { force: errors > 0 };}
                }
                if ui.button("Fermer").clicked(){action=Action::Close;}
            });
        });
        if !open {
            action = Action::Close;
        }
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(
        report: &mut Report,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> (Action, Vec<egui::epaint::ClippedShape>) {
        let mut action = Action::None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200.0, 800.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                action = report.show(ui.ctx());
            },
        );
        output.textures_delta.clear();
        (action, output.shapes)
    }
    fn center(shapes: &[egui::epaint::ClippedShape], label: &str) -> egui::Pos2 {
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
            .unwrap_or_else(|| panic!("Missing {label}"))
    }
    #[test]
    fn report_offers_correction_and_explicit_force_for_configuration_errors() {
        for label in ["Corriger", "Forcer l’export"] {
            let mut report = Report {
                issues: vec![PackIssue {
                    path: "engine_bad.dynx".into(),
                    section: None,
                    parameter: "Power".into(),
                    severity: Severity::Error,
                    message: "Nombre invalide".into(),
                }],
                receiver: None,
                export: true,
                error: None,
            };
            let ctx = egui::Context::default();
            frame(&mut report, &ctx, vec![]);
            let (_, shapes) = frame(&mut report, &ctx, vec![]);
            let pos = center(&shapes, label);
            frame(&mut report, &ctx, vec![egui::Event::PointerMoved(pos)]);
            frame(
                &mut report,
                &ctx,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            let (action, _) = frame(
                &mut report,
                &ctx,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            match action {
                Action::Jump(issue) if label == "Corriger" => assert_eq!(issue.parameter, "Power"),
                Action::Export { force: true } if label == "Forcer l’export" => {}
                _ => panic!("Wrong report action"),
            }
        }
    }
}
