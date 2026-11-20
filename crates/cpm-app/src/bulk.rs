use cpm_core::{BulkOperation, BulkPlan, BulkRequest, Document, Pack, ParameterCatalog};
use eframe::egui;
use std::path::PathBuf;
use std::{
    collections::BTreeMap,
    sync::{mpsc, Arc},
};
pub enum Action {
    None,
    Close,
    Apply(BulkPlan),
}
#[derive(Default)]
pub struct Bulk {
    request: BulkRequest,
    filter: String,
    category: String,
    section_text: String,
    plan: Option<(String, BulkPlan)>,
    worker: Option<mpsc::Receiver<Result<(String, BulkPlan), String>>>,
    pub error: Option<String>,
}
impl Bulk {
    fn key(&self) -> String {
        format!("{:?}", self.request)
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        pack: &Pack,
        documents: BTreeMap<PathBuf, Document>,
        catalog: Arc<ParameterCatalog>,
    ) -> Action {
        if let Some(rx) = &self.worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.worker = None;
                    match result {
                        Ok(p) => self.plan = Some(p),
                        Err(e) => self.error = Some(e),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.worker = None;
                    self.error = Some("Préparation interrompue".into());
                }
                _ => {}
            }
        }
        if self
            .plan
            .as_ref()
            .is_some_and(|(key, _)| *key != self.key())
        {
            self.plan = None;
        }
        let mut action = Action::None;
        let mut open = true;
        let mut prepare = false;
        let mut apply = false;
        egui::Window::new("Édition groupée des configurations").open(&mut open).default_size([760.,650.]).max_height(680.0).vscroll(true).collapsible(false).show(ctx,|ui|{
            ui.label("Sélectionner les fichiers, préparer les changements puis vérifier avant application. Les fichiers source ne sont modifiés qu’à l’export.");
            ui.add_enabled_ui(self.worker.is_none(),|ui|{
                ui.horizontal(|ui|{ui.label("Rechercher");ui.text_edit_singleline(&mut self.filter);ui.label("Catégorie");ui.text_edit_singleline(&mut self.category).on_hover_text("Préfixe : vehicle, engine, wheel, armor… Vide pour tout afficher.");});
                let paths:Vec<_>=pack.configs().iter().filter(|p|p.to_string_lossy().to_lowercase().contains(&self.filter.to_lowercase())&&(self.category.trim().is_empty()||p.file_name().unwrap_or_default().to_string_lossy().to_lowercase().starts_with(&format!("{}_",self.category.trim().to_lowercase())))).cloned().collect();
                ui.horizontal(|ui|{if ui.button("Sélectionner les fichiers affichés").clicked(){self.request.paths.extend(paths.iter().cloned());}
                if ui.button("Tout désélectionner").clicked(){self.request.paths.clear();}ui.label(format!("{} sélectionné(s)",self.request.paths.len()));});
                egui::ScrollArea::vertical().id_salt("bulk-paths").max_height(150.).show(ui,|ui|{for path in paths {let mut checked=self.request.paths.contains(&path);if ui.checkbox(&mut checked,path.display().to_string()).changed(){if checked{self.request.paths.insert(path);}else{self.request.paths.remove(&path);}}}});
                ui.horizontal(|ui|{ui.label("Toutes les sections de ce nom (vide = Racine)");ui.text_edit_singleline(&mut self.section_text);});self.request.section=(!self.section_text.trim().is_empty()).then(||self.section_text.trim().to_owned());
                ui.horizontal(|ui|{ui.label("Opération");ui.selectable_value(&mut self.request.operation,BulkOperation::ReplaceExisting,"Modifier les présents");ui.selectable_value(&mut self.request.operation,BulkOperation::AddMissing,"Ajouter les absents");ui.selectable_value(&mut self.request.operation,BulkOperation::Remove,"Supprimer");});
                let definitions=self.request.paths.iter().next().and_then(|p|{let doc=documents.get(p).cloned().map(Ok).unwrap_or_else(||pack.read_config(pack.configs().iter().position(|x|x==p).unwrap()));doc.ok().map(|d|{let section=self.request.section.as_ref().and_then(|n|d.sections().iter().position(|s|s.name==*n));catalog.parameters(p,&d,section)})}).unwrap_or_default();
                ui.horizontal(|ui|{ui.label("Paramètre");ui.text_edit_singleline(&mut self.request.parameter);egui::ComboBox::from_id_salt("bulk-definitions").selected_text("Paramètres du premier fichier…").show_ui(ui,|ui|{for d in &definitions{if ui.selectable_label(false,&d.name).on_hover_text(&d.description).clicked(){self.request.parameter=d.name.clone();self.request.value=d.default_value.clone().unwrap_or_default();}}});});
                if self.request.operation!=BulkOperation::Remove {ui.horizontal(|ui|{ui.label("Nouvelle valeur");crate::parameter_input::input(ui,&mut self.request.value,definitions.iter().find(|d|d.name==self.request.parameter),None);});}
                if ui.button("Préparer les modifications").clicked(){prepare=true;}
                if let Some((key,plan))=&self.plan {if *key==self.key(){
                    ui.separator();ui.label(format!("{} fichier(s) seront modifiés",plan.documents.len()));
                    egui::ScrollArea::vertical().id_salt("bulk-summary").max_height(220.).show(ui,|ui|{for doc in &plan.documents {ui.collapsing(doc.path.display().to_string(),|ui|{for change in cpm_core::parameter_changes(Some(&doc.before),Some(&doc.after)){ui.label(format!("{} : {} → {}",change.parameter,change.before.as_deref().unwrap_or("(absent)"),change.after.as_deref().unwrap_or("(absent)")));}});}for warning in &plan.warnings{ui.colored_label(crate::theme::WARNING,warning);}});
                    if ui.add_enabled(!plan.documents.is_empty(),egui::Button::new("Appliquer ces modifications aux fichiers sélectionnés")).clicked(){apply=true;}
                }}
            });
            if self.worker.is_some(){ui.spinner();ctx.request_repaint_after(std::time::Duration::from_millis(100));}
                if let Some(e)=&self.error {ui.colored_label(crate::theme::ERROR,e);}
        });
        if !open {
            return Action::Close;
        }
        if prepare {
            let request = self.request.clone();
            let key = self.key();
            let pack = pack.clone();
            let (tx, rx) = mpsc::channel();
            self.worker = Some(rx);
            self.plan = None;
            self.error = None;
            std::thread::spawn(move || {
                let _ = tx.send(
                    pack.plan_bulk_edit(&request, &documents, &catalog)
                        .map(|p| (key, p)),
                );
            });
        }
        if apply {
            if let Some((key, plan)) = self.plan.take() {
                if key == self.key() {
                    action = Action::Apply(plan);
                }
            }
        }
        action
    }
}
