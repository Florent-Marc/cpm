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
    Export,
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
            if let Some(error)=&self.error{ui.colored_label(egui::Color32::LIGHT_RED,error);}
            egui::ScrollArea::vertical().max_height(340.0).show(ui,|ui| {
                for issue in &self.issues {
                    let color=if issue.severity==Severity::Error{egui::Color32::LIGHT_RED}else{egui::Color32::YELLOW};
                    if ui.button(egui::RichText::new(format!("{} · {} : {}",issue.path.display(),issue.parameter,issue.message)).color(color)).clicked(){action=Action::Jump(issue.clone());}
                }
            });
            ui.horizontal(|ui| {
                if self.export && ui.add_enabled(self.receiver.is_none() && errors==0 && self.error.is_none(),egui::Button::new("Continuer vers l’export")).clicked(){action=Action::Export;}
                if ui.button("Fermer").clicked(){action=Action::Close;}
            });
        });
        if !open {
            action = Action::Close;
        }
        action
    }
}
