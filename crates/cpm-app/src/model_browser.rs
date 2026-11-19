use cpm_core::{Document, Pack};
use eframe::egui;
use std::{collections::BTreeMap, path::PathBuf, sync::mpsc};
pub enum Selection {
    Config(usize),
    Asset(PathBuf),
}
pub struct Browser {
    receiver: Option<mpsc::Receiver<BTreeMap<PathBuf, Vec<(usize, PathBuf)>>>>,
    models: BTreeMap<PathBuf, Vec<(usize, PathBuf)>>,
    filter: String,
    error: Option<String>,
}
impl Browser {
    pub fn new(pack: Pack, documents: BTreeMap<PathBuf, Document>) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut models: BTreeMap<_, Vec<_>> = pack
                .obj_resources()
                .into_iter()
                .map(|p| (p, Vec::new()))
                .collect();
            for (index, path) in pack.configs().iter().enumerate() {
                let doc = documents
                    .get(path)
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(|| pack.read_config(index));
                if let Ok(doc) = doc {
                    if let Some(model) = doc
                        .properties()
                        .iter()
                        .find(|p| p.section.is_none() && p.key == "Model")
                    {
                        if let Ok(asset) = pack.resolve_asset(path, &model.value) {
                            if asset
                                .extension()
                                .is_some_and(|e| e.eq_ignore_ascii_case("obj"))
                            {
                                models.entry(asset).or_default().push((index, path.clone()));
                            }
                        }
                    }
                }
            }
            let _ = tx.send(models);
        });
        Self {
            receiver: Some(rx),
            models: BTreeMap::new(),
            filter: String::new(),
            error: None,
        }
    }
    pub fn show(&mut self, ctx: &egui::Context) -> (bool, Option<Selection>) {
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok(models) => {
                    self.models = models;
                    self.receiver = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("Indexation interrompue".into());
                    self.receiver = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100))
                }
            }
        }
        let mut open = true;
        let mut selected = None;
        egui::Window::new("Modèles OBJ du pack").open(&mut open).default_size([500.0,420.0]).show(ctx,|ui|{
            ui.text_edit_singleline(&mut self.filter);ui.weak("Seul le modèle choisi est chargé en 3D. Les configurations assemblent aussi leurs roues et leur volant.");
            if self.receiver.is_some(){ui.spinner();}
            if let Some(error)=&self.error{ui.colored_label(egui::Color32::LIGHT_RED,error);}
            let query=self.filter.to_lowercase();
            egui::ScrollArea::vertical().show(ui,|ui|{
                for (path,configs) in &self.models {
                    if !path.to_string_lossy().to_lowercase().contains(&query)&&!configs.iter().any(|(_,p)|p.to_string_lossy().to_lowercase().contains(&query)){continue;}
                    egui::CollapsingHeader::new(path.file_name().unwrap_or_default().to_string_lossy()).id_salt(path).show(ui,|ui|{
                        ui.weak(path.display().to_string());
                        if ui.button("Aperçu OBJ seul").clicked(){selected=Some(Selection::Asset(path.clone()));}
                        for (index,config) in configs {if ui.button(crate::navigation::config_label(config)).on_hover_text(config.display().to_string()).clicked(){selected=Some(Selection::Config(*index));}}
                    });
                }
            });
        });
        (open, selected)
    }
}
