use super::{picking, View};
use cpm_core::{Document, ObjMesh};
use eframe::egui::{self, Pos2, Rect, Vec2};
#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Tool {
    #[default]
    Orbit,
    Move,
    Resize,
}
pub struct Edit {
    pub section: usize,
    pub key: String,
    pub value: String,
    pub expected_source: String,
}
#[derive(Default)]
pub(super) struct Gizmo {
    pub tool: Tool,
    pub snap: bool,
    pub step: f64,
    drag: Option<Drag>,
}
struct Drag {
    section: usize,
    axis: usize,
    start: Pos2,
    origin: [String; 3],
    values: [f64; 3],
    unit: Vec2,
    source: String,
    key: String,
    current: f64,
    resize: bool,
}
struct Control {
    position: [f64; 3],
    scale: [f64; 3],
    key: String,
    values: [f64; 3],
    tokens: [String; 3],
}
fn property<'a>(
    document: &'a Document,
    section: Option<usize>,
    names: &[&str],
) -> Option<(&'a str, &'a str)> {
    names.iter().find_map(|name| {
        document
            .properties()
            .iter()
            .find(|p| p.section == section && p.key.eq_ignore_ascii_case(name))
            .map(|p| (p.key.as_str(), p.value.as_str()))
    })
}
fn vector(value: &str) -> Option<([f64; 3], [String; 3])> {
    let tokens: Vec<_> = value.split_whitespace().collect();
    if tokens.len() != 3 {
        return None;
    }
    let mut numbers = [0.0_f64; 3];
    for i in 0..3 {
        numbers[i] = tokens[i].parse().ok()?;
        if !numbers[i].is_finite() {
            return None;
        }
    }
    Some((numbers, std::array::from_fn(|i| tokens[i].to_owned())))
}
fn control(document: &Document, section: usize, resize: bool) -> Result<Control, &'static str> {
    let info = document
        .sections()
        .get(section)
        .ok_or("Sélectionner une section")?;
    let name = info.name.to_ascii_lowercase();
    if info.parent.is_some()
        || !["wheel", "steeringwheel", "shape"]
            .iter()
            .any(|p| name.starts_with(p))
    {
        return Err("Choisir une roue, un volant ou une boîte de collision");
    }
    if resize && !name.starts_with("shape") {
        return Err("Le redimensionnement concerne les boîtes Shape…");
    }
    if resize
        && property(document, Some(section), &["Type"])
            .is_some_and(|(_, t)| !t.eq_ignore_ascii_case("BOX"))
    {
        return Err("Le redimensionnement concerne les boîtes BOX");
    }
    let (position, _) = property(document, Some(section), &["Position", "ShapePosition"])
        .and_then(|(_, v)| vector(v))
        .ok_or("Renseigner une Position explicite et valide avant de déplacer cette pièce")?;
    let (scale, _) = property(document, None, &["ScaleModifier"])
        .map(|(_, v)| vector(v))
        .unwrap_or_else(|| vector("1 1 1"))
        .ok_or("ScaleModifier invalide")?;
    if scale.iter().any(|v| v.abs() < 1e-12) {
        return Err("ScaleModifier ne peut pas contenir zéro");
    }
    let (key, text) = if resize {
        property(
            document,
            Some(section),
            &["Scale", "Size", "ShapeScale", "BoxDim"],
        )
        .unwrap_or(("Scale", "0.5 1 0.5"))
    } else {
        property(document, Some(section), &["Position", "ShapePosition"])
            .ok_or("Position absente")?
    };
    let (values, tokens) = vector(text).ok_or("Valeur numérique invalide")?;
    Ok(Control {
        position,
        scale,
        key: key.into(),
        values,
        tokens,
    })
}
pub(super) fn config_world(p: [f64; 3], scale: [f64; 3]) -> [f64; 3] {
    [p[0] * scale[0], p[2] * scale[1], -p[1] * scale[2]]
}
fn change(start: f64, delta: Vec2, unit: Vec2, snap: Option<f64>, resize: bool) -> f64 {
    let value = start + delta.dot(unit) as f64 / unit.length_sq() as f64;
    let value = if let Some(step) = snap {
        (value / step).round() * step
    } else {
        value
    };
    if resize {
        value.max(0.001)
    } else {
        value
    }
}
impl Gizmo {
    pub fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Orbit, "Tourner");
            ui.selectable_value(&mut self.tool, Tool::Move, "Déplacer");
            ui.selectable_value(&mut self.tool, Tool::Resize, "Taille (boîte)");
            ui.checkbox(&mut self.snap, "Grille");
            if self.step <= 0.0 {
                self.step = 0.1;
            }
            ui.add(
                egui::DragValue::new(&mut self.step)
                    .range(0.001..=1000.0)
                    .speed(0.01),
            );
        });
    }
    pub fn preview_document(&self, document: &Document) -> Option<Document> {
        let drag = self.drag.as_ref()?;
        let mut tokens = drag.origin.clone();
        tokens[drag.axis] = drag.current.to_string();
        let mut draft = document.clone();
        if let Some(index) = draft
            .properties()
            .iter()
            .position(|p| p.section == Some(drag.section) && p.key == drag.key)
        {
            draft.set_value(index, &tokens.join(" ")).ok()?;
        } else {
            draft
                .add_property(Some(drag.section), &drag.key, &tokens.join(" "))
                .ok()?;
        }
        Some(draft)
    }
    pub fn active(&self) -> bool {
        self.drag.is_some()
    }
    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        mesh: &ObjMesh,
        document: &Document,
        section: Option<usize>,
        view: View,
        rect: Rect,
    ) -> Option<Edit> {
        if self.tool == Tool::Orbit {
            self.drag = None;
            return None;
        }
        let Some(section) = section else {
            self.drag = None;
            ui.painter().text(
                rect.left_top() + egui::vec2(12.0, 12.0),
                egui::Align2::LEFT_TOP,
                "Sélectionner une roue, un volant ou une boîte",
                egui::FontId::proportional(13.0),
                egui::Color32::WHITE,
            );
            return None;
        };
        let resize = self.tool == Tool::Resize;
        let mut control = match control(document, section, resize) {
            Ok(c) => c,
            Err(error) => {
                self.drag = None;
                ui.painter().text(
                    rect.left_top() + egui::vec2(12.0, 12.0),
                    egui::Align2::LEFT_TOP,
                    error,
                    egui::FontId::proportional(13.0),
                    egui::Color32::YELLOW,
                );
                return None;
            }
        };
        if let Some(drag) = self
            .drag
            .as_ref()
            .filter(|d| d.section == section && !d.resize)
        {
            control.position[drag.axis] = drag.current;
        }
        let world = config_world(control.position, control.scale);
        let origin = picking::project(mesh.normalize_point(world), view, rect).0;
        let colors = [
            egui::Color32::LIGHT_RED,
            egui::Color32::LIGHT_GREEN,
            egui::Color32::LIGHT_BLUE,
        ];
        let mut edit = None;
        for axis in 0..3 {
            let mut other = control.position;
            other[axis] += 1.0;
            let unit = picking::project(
                mesh.normalize_point(config_world(other, control.scale)),
                view,
                rect,
            )
            .0 - origin;
            if unit.length_sq() < 0.001 {
                continue;
            }
            let endpoint = origin + unit.normalized() * 75.0;
            ui.painter().arrow(
                origin,
                endpoint - origin,
                egui::Stroke::new(3.0, colors[axis]),
            );
            ui.painter().text(
                endpoint,
                egui::Align2::LEFT_BOTTOM,
                ["X", "Y", "Z"][axis],
                egui::FontId::proportional(15.0),
                colors[axis],
            );
            let response = ui.interact(
                Rect::from_center_size(endpoint, Vec2::splat(24.0)).intersect(rect),
                ui.scope_id().with(("axis-handle", section, axis)),
                egui::Sense::drag(),
            );
            if response.drag_started() {
                self.drag = Some(Drag {
                    section,
                    axis,
                    start: response.interact_pointer_pos().unwrap_or(endpoint)
                        - response.total_drag_delta().unwrap_or_default(),
                    origin: control.tokens.clone(),
                    values: control.values,
                    unit,
                    source: document.source().into(),
                    key: control.key.clone(),
                    current: control.values[axis],
                    resize,
                });
            }
            if let Some(drag) = self
                .drag
                .as_mut()
                .filter(|d| d.section == section && d.axis == axis)
            {
                if response.dragged() || response.drag_stopped() {
                    drag.current = change(
                        drag.values[axis],
                        response
                            .interact_pointer_pos()
                            .or_else(|| ui.input(|i| i.pointer.latest_pos()))
                            .unwrap_or(drag.start)
                            - drag.start,
                        drag.unit,
                        self.snap.then_some(self.step),
                        drag.resize,
                    );
                }
                ui.painter().text(
                    rect.left_bottom() + egui::vec2(12.0, -12.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!(
                        "{} {} = {:.4} · relâcher pour appliquer",
                        drag.key,
                        ["X", "Y", "Z"][axis],
                        drag.current
                    ),
                    egui::FontId::proportional(14.0),
                    colors[axis],
                );
                if response.drag_stopped() {
                    if drag.current.is_finite() && (drag.current - drag.values[axis]).abs() > 1e-9 {
                        let mut tokens = drag.origin.clone();
                        let value = format!("{:.6}", drag.current);
                        tokens[axis] = value.trim_end_matches('0').trim_end_matches('.').to_owned();
                        if tokens[axis] == "-0" {
                            tokens[axis] = "0".into();
                        }
                        edit = Some(Edit {
                            section,
                            key: drag.key.clone(),
                            value: tokens.join(" "),
                            expected_source: drag.source.clone(),
                        });
                    }
                    self.drag = None;
                }
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.drag = None;
            edit = None;
        }
        edit
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn movement_projects_units_snaps_and_preserves_positive_half_extents() {
        assert_eq!(
            config_world([1.0, 2.0, 3.0], [2.0, 3.0, 4.0]),
            [2.0, 9.0, -8.0]
        );
        let unit = Vec2::new(20.0, -10.0);
        assert!((change(2.0, unit * 1.5, unit, None, false) - 3.5).abs() < 1e-6);
        assert!((change(2.0, unit * 0.24, unit, Some(0.5), false) - 2.0).abs() < 1e-6);
        assert_eq!(change(1.0, unit * (-5.0), unit, None, true), 0.001);
        let d = Document::parse("Shape_body{\nShapePosition: 1 2 3\nBoxDim: 4 5 6\n}\n").unwrap();
        let c = control(&d, 0, true).unwrap();
        assert_eq!(c.key, "BoxDim");
        assert_eq!(c.tokens, ["4", "5", "6"]);
        let automatic = Document::parse("SteeringWheel{\n}\n").unwrap();
        assert!(control(&automatic, 0, false).is_err());
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    fn frame(
        ctx: &egui::Context,
        gizmo: &mut Gizmo,
        mesh: &ObjMesh,
        doc: &Document,
        events: Vec<egui::Event>,
    ) -> Option<Edit> {
        let mut edit = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0))),
                events,
                ..Default::default()
            },
            |ui| {
                edit = gizmo.draw(
                    ui,
                    mesh,
                    doc,
                    Some(0),
                    View {
                        yaw: 0.0,
                        pitch: 0.0,
                        zoom: 1.0,
                        pan: Vec2::ZERO,
                    },
                    Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::splat(400.0)),
                );
            },
        );
        output.textures_delta.clear();
        edit
    }
    #[test]
    fn pointer_drag_accumulates_motion_and_commits_only_on_release() {
        let ctx = egui::Context::default();
        let mut gizmo = Gizmo {
            tool: Tool::Move,
            ..Default::default()
        };
        let mesh = ObjMesh::parse(b"v -1 -1 0\nv 1 -1 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        let doc = Document::parse("Shape_body{\nPosition: 0 0 0\nScale: 1 1 1\n}\n").unwrap();
        let endpoint = Pos2::new(295.0, 220.0);
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(&ctx, &mut gizmo, &mesh, &doc, vec![]);
        frame(
            &ctx,
            &mut gizmo,
            &mesh,
            &doc,
            vec![egui::Event::PointerMoved(endpoint), button(endpoint, true)],
        );
        assert!(frame(
            &ctx,
            &mut gizmo,
            &mesh,
            &doc,
            vec![egui::Event::PointerMoved(endpoint + Vec2::new(20.0, 0.0))]
        )
        .is_none());
        assert!(frame(
            &ctx,
            &mut gizmo,
            &mesh,
            &doc,
            vec![egui::Event::PointerMoved(endpoint + Vec2::new(40.0, 0.0))]
        )
        .is_none());
        let live = gizmo
            .preview_document(&doc)
            .expect("live placement during drag");
        assert_ne!(live.source(), doc.source());
        assert_eq!(doc.properties()[0].value, "0 0 0");
        let wires = mesh.preview_collisions(&live);
        assert_eq!(wires.len(), 1);
        assert_ne!(wires[0].vertices, mesh.preview_collisions(&doc)[0].vertices);
        let edit = frame(
            &ctx,
            &mut gizmo,
            &mesh,
            &doc,
            vec![button(endpoint + Vec2::new(40.0, 0.0), false)],
        )
        .expect("one edit on release");
        let x = edit
            .value
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!((x - 0.282843).abs() < 0.0001, "{x}");
        assert_eq!(edit.key, "Position");
        assert_eq!(edit.expected_source, doc.source());
        assert!(!gizmo.active());
    }
}
