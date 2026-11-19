use cpm_core::{Document, Pack};
use eframe::egui;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
};
#[derive(Clone)]
pub struct Hit {
    pub index: usize,
    pub section: Option<usize>,
    pub text: String,
}
#[derive(Default)]
pub struct Search {
    source: Option<PathBuf>,
    pub open: bool,
    query: String,
    results: Vec<Hit>,
    receiver: Option<mpsc::Receiver<Vec<Hit>>>,
    cancel: Arc<AtomicBool>,
    changed_at: Option<f64>,
    searched: String,
}
impl Drop for Search {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Search {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        pack: Option<&Pack>,
        documents: BTreeMap<PathBuf, Document>,
    ) -> Option<Hit> {
        let source = pack.map(|p| p.source().to_owned());
        if source != self.source {
            self.cancel.store(true, Ordering::Relaxed);
            self.receiver = None;
            self.results.clear();
            self.searched.clear();
            self.changed_at = Some(ctx.input(|i| i.time) - 0.3);
            self.source = source;
        }
        if !self.open {
            return None;
        }
        if let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(hits) => {
                    self.results = hits;
                    self.receiver = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => self.receiver = None,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        let mut open = self.open;
        let mut chosen = None;
        egui::Window::new("Rechercher dans le pack")
            .open(&mut open)
            .default_size([650.0, 350.0])
            .show(ctx, |ui| {
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Nom de fichier, section, paramètre ou valeur…")
                            .desired_width(f32::INFINITY),
                    )
                    .changed()
                {
                    self.changed_at = Some(ui.input(|i| i.time));
                    self.cancel.store(true, Ordering::Relaxed);
                    self.receiver = None;
                    self.results.clear();
                }
                if pack.is_none() {
                    ui.label("Ouvrir un pack pour rechercher dans ses configurations.");
                }
                if self.receiver.is_some() {
                    ui.spinner();
                }
                ui.label(format!("{} résultat(s) · maximum 500", self.results.len()));
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for hit in &self.results {
                        if ui.button(&hit.text).clicked() {
                            chosen = Some(hit.clone());
                        }
                    }
                });
            });
        self.open = open;
        let now = ctx.input(|i| i.time);
        if self.changed_at.is_some_and(|at| now - at >= 0.3)
            || (self.searched != self.query && self.changed_at.is_none())
        {
            self.changed_at = None;
            self.searched = self.query.clone();
            if let Some(pack) = pack.filter(|_| !self.query.trim().is_empty()) {
                let pack = pack.clone();
                let query = self.query.clone();
                let (tx, rx) = mpsc::channel();
                self.receiver = Some(rx);
                self.cancel = Arc::new(AtomicBool::new(false));
                let cancel = self.cancel.clone();
                std::thread::spawn(move || {
                    let mut hits = Vec::new();
                    for (index, path) in pack.configs().iter().enumerate() {
                        if cancel.load(Ordering::Relaxed) {
                            return;
                        }
                        let document = documents
                            .get(path)
                            .cloned()
                            .map(Ok)
                            .unwrap_or_else(|| pack.read_config(index));
                        if let Ok(document) = document {
                            hits.extend(find(&document, &query).into_iter().map(
                                |(section, text)| Hit {
                                    index,
                                    section,
                                    text: format!("{} · {text}", path.display()),
                                },
                            ));
                        }
                        if path
                            .to_string_lossy()
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                        {
                            hits.push(Hit {
                                index,
                                section: None,
                                text: path.display().to_string(),
                            });
                        }
                        if hits.len() >= 500 {
                            hits.truncate(500);
                            break;
                        }
                    }
                    let _ = tx.send(hits);
                });
            } else {
                self.receiver = None;
                self.results.clear();
            }
        }
        if self.changed_at.is_some() || self.receiver.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        chosen
    }
}
fn find(document: &Document, query: &str) -> Vec<(Option<usize>, String)> {
    let query = query.to_lowercase();
    let mut hits = Vec::new();
    for (index, section) in document.sections().iter().enumerate() {
        if section.name.to_lowercase().contains(&query) {
            hits.push((
                Some(index),
                format!("{} · ligne {}", section.name, section.line),
            ));
        }
    }
    for p in document.properties() {
        if p.key.to_lowercase().contains(&query) || p.value.to_lowercase().contains(&query) {
            hits.push((
                p.section,
                format!("{}: {} · ligne {}", p.key, p.value, p.line),
            ));
        }
    }
    hits.truncate(500);
    hits
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn searches_unicode_values_keys_and_sections() {
        let d = Document::parse("Name: Véhicule\nWheel_left{\nPosition: 1 2 3\n}\n").unwrap();
        assert_eq!(find(&d, "VÉHICULE")[0].0, None);
        assert_eq!(find(&d, "position")[0].0, Some(0));
        assert_eq!(find(&d, "wheel").len(), 1);
    }
}
