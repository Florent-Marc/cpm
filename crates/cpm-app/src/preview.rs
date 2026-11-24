mod gizmo;
mod picking;
use cpm_core::{ObjMesh, ObjRequest};
use eframe::{egui, egui_glow, glow};
pub use gizmo::Edit as GizmoEdit;
use glow::HasContext as _;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Condvar, Mutex, OnceLock,
};

pub struct Preview {
    detached: bool,
    close_requested: bool,
    #[cfg(test)]
    scene_rect: egui::Rect,
    cancel: Arc<AtomicBool>,
    pub interactive: bool,
    pub live: bool,
    document: cpm_core::Document,
    collision_source: String,
    collision_wires: Vec<cpm_core::CollisionWire>,
    gizmo: gizmo::Gizmo,
    edits: Vec<GizmoEdit>,
    pub variant: usize,
    pub reload_requested: bool,
    variant_names: Vec<String>,
    pub selected: Option<usize>,
    selection_changed: Option<Option<usize>>,
    geometry: Option<Arc<ObjMesh>>,
    path: String,
    receiver: Option<mpsc::Receiver<Result<ObjMesh, String>>>,
    mesh: Option<Arc<GpuMesh>>,
    error: Option<String>,
    statistics: String,
    parts: Vec<PartVisibility>,
    warnings: Vec<String>,
    use_textures: bool,
    show_collisions: bool,
    collision_names: Vec<String>,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    pan: egui::Vec2,
}
impl Preview {
    pub fn open(request: ObjRequest, ctx: &egui::Context) -> Self {
        let document = request.document().clone();
        let variant_names = request.variant_names();
        let path = request.path.display().to_string();
        let (receiver, cancel) = background(request, ctx);
        Self {
            detached: true,
            close_requested: false,
            #[cfg(test)]
            scene_rect: egui::Rect::NOTHING,
            cancel,
            interactive: true,
            live: true,
            document,
            collision_source: String::new(),
            collision_wires: Vec::new(),
            gizmo: gizmo::Gizmo::default(),
            edits: Vec::new(),
            variant: 0,
            reload_requested: false,
            variant_names,
            selected: None,
            selection_changed: None,
            geometry: None,
            path,
            receiver: Some(receiver),
            mesh: None,
            error: None,
            statistics: String::new(),
            parts: Vec::new(),
            warnings: Vec::new(),
            use_textures: true,
            show_collisions: false,
            collision_names: Vec::new(),
            yaw: -0.6,
            pitch: 0.25,
            zoom: 0.9,
            pan: egui::Vec2::ZERO,
        }
    }
    pub fn take_edits(&mut self) -> Vec<GizmoEdit> {
        std::mem::take(&mut self.edits)
    }
    pub fn take_selection(&mut self) -> Option<Option<usize>> {
        self.selection_changed.take()
    }
    pub fn manipulating(&self) -> bool {
        self.gizmo.active()
    }
    pub fn sync_document(&mut self, document: cpm_core::Document) {
        self.document = document;
    }
    pub fn busy(&self) -> bool {
        self.receiver.is_some()
    }
    pub fn refresh(&mut self, request: ObjRequest, ctx: &egui::Context) {
        self.document = request.document().clone();
        self.variant_names = request.variant_names();
        if self.variant >= self.variant_names.len() {
            self.variant = 0;
        }
        self.reload_requested = false;
        self.path = request.path.display().to_string();
        self.error = None;
        self.cancel.store(true, Ordering::Relaxed);
        let (receiver, cancel) = background(request, ctx);
        self.receiver = Some(receiver);
        self.cancel = cancel;
    }
    pub fn fail(&mut self, error: String) {
        self.error = Some(error);
        self.mesh = None;
        self.geometry = None;
    }
    pub fn show(&mut self, ctx: &egui::Context, gl: Option<&Arc<glow::Context>>) -> bool {
        if let Some(receiver) = self.receiver.as_ref().filter(|_| !self.gizmo.active()) {
            match receiver.try_recv() {
                Ok(result) => {
                    self.receiver = None;
                    let result = result.and_then(|mesh| {
                        self.statistics = format!(
                            "{} triangles · {} pièces · {} textures",
                            mesh.triangles,
                            mesh.objects,
                            mesh.textures.len()
                        );
                        let previous: std::collections::BTreeMap<_, _> = self
                            .parts
                            .iter()
                            .map(|p| (p.name.clone(), p.visible))
                            .collect();
                        self.parts = mesh
                            .parts
                            .iter()
                            .map(|p| {
                                let materials: std::collections::BTreeSet<_> = p
                                    .batches
                                    .iter()
                                    .filter_map(|b| {
                                        b.material
                                            .and_then(|i| mesh.materials.get(i))
                                            .map(|m| m.name.as_str())
                                    })
                                    .collect();
                                PartVisibility {
                                    section: p.section,
                                    name: p.name.clone(),
                                    visible: previous.get(&p.name).copied().unwrap_or(true),
                                    details: format!(
                                        "{} triangles · Matériaux : {}",
                                        p.batches.iter().map(|b| b.count / 3).sum::<usize>(),
                                        materials.into_iter().collect::<Vec<_>>().join(", ")
                                    ),
                                }
                            })
                            .collect();
                        self.collision_names =
                            mesh.collisions.iter().map(|c| c.name.clone()).collect();
                        self.warnings = mesh.warnings.clone();
                        let gl = gl.ok_or("Le moteur OpenGL de l’aperçu n’est pas disponible")?;
                        let gpu = GpuMesh::new(gl.clone(), &mesh)?;
                        self.collision_source.clear();
                        self.geometry = Some(Arc::new(mesh));
                        Ok(gpu)
                    });
                    match result {
                        Ok(mesh) => self.mesh = Some(Arc::new(mesh)),
                        Err(error) => self.fail(error),
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100))
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.receiver = None;
                    self.error = Some("Le chargement du modèle a été interrompu".into());
                }
            }
        }
        let mut open = true;
        if self.detached {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("cpm-model-preview"),
                egui::ViewportBuilder::default()
                    .with_title("CPM — Aperçu 3D")
                    .with_inner_size([1100.0, 760.0])
                    .with_min_inner_size([640.0, 400.0]),
                |ui, _class| {
                    if ui.ctx().input(|i| i.viewport().close_requested()) {
                        open = false;
                    }
                    egui::CentralPanel::default().show(ui, |ui| self.contents(ui));
                },
            );
        } else {
            egui::Window::new("Aperçu 3D — modèle")
                .open(&mut open)
                .default_size([1000.0, 700.0])
                .min_size([600.0, 400.0])
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| self.contents(ui));
        }
        open && !self.close_requested
    }
    fn contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let label = if self.detached {
                "Intégrer à CPM"
            } else {
                "Détacher la fenêtre…"
            };
            if ui
                .button(label)
                .on_hover_text("Une fenêtre séparée peut être déplacée sur un autre écran")
                .clicked()
            {
                self.detached = !self.detached;
                ui.ctx().request_repaint();
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
            if ui.button("Fermer l’aperçu").clicked() {
                self.close_requested = true;
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
            ui.add(egui::Label::new(&self.path).truncate())
                .on_hover_text(&self.path);
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("Recentrer").clicked() {
                self.yaw = -0.6;
                self.pitch = 0.25;
                self.zoom = 0.9;
                self.pan = egui::Vec2::ZERO;
            }
            if ui.button("Face").clicked() {
                self.yaw = 0.0;
                self.pitch = 0.0;
            }
            if ui.button("Côté").clicked() {
                self.yaw = std::f32::consts::FRAC_PI_2;
                self.pitch = 0.0;
            }
            if ui.button("Dessus").clicked() {
                self.yaw = 0.0;
                self.pitch = std::f32::consts::FRAC_PI_2;
            }
            ui.label(&self.statistics);
        });
        ui.add_enabled_ui(self.interactive, |ui| self.gizmo.toolbar(ui));
        ui.small("Clic : sélectionner · Glisser hors poignées : tourner · Molette : zoomer · Bouton droit : déplacer");

        egui::Panel::left("preview-options").default_size(260.0).min_size(200.0).max_size((ui.available_width()*0.48).max(200.0)).resizable(true).show(ui, |ui| {
            egui::ScrollArea::vertical().id_salt("preview-controls").show(ui, |ui| {
            ui.checkbox(&mut self.live,"Actualisation automatique");
            if self.variant_names.len()>1 {
                let before=self.variant;
                egui::ComboBox::from_id_salt("texture-variant").selected_text(&self.variant_names[self.variant]).show_ui(ui,|ui|{
                    for (index,name) in self.variant_names.iter().enumerate(){ui.selectable_value(&mut self.variant,index,name);}
                });
                if before!=self.variant{self.reload_requested=true;ui.ctx().request_repaint();}
            }
            ui.vertical(|ui| {
                ui.checkbox(&mut self.use_textures,"Afficher les textures");
                ui.checkbox(&mut self.show_collisions,format!("Afficher les collisions ({})",self.collision_names.len())).on_hover_text(format!("Boîtes Shape… en fil de fer, visibles à travers le modèle. Les collisions générées automatiquement ne sont pas affichées.\n{}",self.collision_names.join("\n")));
            });
            if !self.parts.is_empty() {
                egui::CollapsingHeader::new(format!("Pièces ({})",self.parts.len())).default_open(true).show(ui,|ui| {
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Tout afficher").clicked(){for p in &mut self.parts{p.visible=true;}}
                        if ui.button("Tout masquer").clicked(){for p in &mut self.parts{p.visible=false;}}
                    });
                    let mut isolate=None;
                    egui::ScrollArea::vertical().id_salt("obj-parts").show(ui,|ui| {
                        let mut groups:std::collections::BTreeMap<String,Vec<usize>>=std::collections::BTreeMap::new();
                        for (index,part) in self.parts.iter().enumerate(){groups.entry(part.name.split_once('/').map(|(group,_)|group.to_owned()).unwrap_or("Modèle principal".into())).or_default().push(index);}
                        for (group,indices) in groups {
                            egui::CollapsingHeader::new(group).default_open(true).show(ui,|ui|{
                                for index in indices {
                                    let part=&mut self.parts[index];
                                    ui.push_id(index,|ui|{ui.horizontal(|ui|{
                                        ui.checkbox(&mut part.visible,"");
                                        if ui.add(egui::Button::selectable(part.section.is_some()&&self.selected==part.section,part.name.split_once('/').map(|(_,name)|name).unwrap_or(&part.name)).truncate()).on_hover_text(&part.details).clicked(){self.selected=part.section;self.selection_changed=Some(part.section);}
                                        if ui.small_button("Isoler").clicked(){isolate=Some(index);}
                                    });});
                                }
                            });
                        }
                    });
                    if let Some(selected)=isolate {for (index,part) in self.parts.iter_mut().enumerate(){part.visible=index==selected;}}
                });
            }
            if !self.warnings.is_empty(){
                ui.collapsing(format!("Avertissements ({})",self.warnings.len()),|ui|{
                    egui::ScrollArea::vertical().id_salt("obj-warnings").max_height(100.0).show(ui,|ui|{for warning in &self.warnings{ui.colored_label(egui::Color32::YELLOW,warning);}});
                });
            }
            if self.receiver.is_some() { ui.spinner(); ui.label("Chargement du modèle…"); }
            if let Some(error) = &self.error { ui.colored_label(egui::Color32::LIGHT_RED,error); }

            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            let size = egui::vec2(
                ui.available_width().max(1.0),
                ui.available_height().max(1.0),
            );
            let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
            #[cfg(test)]
            {
                self.scene_rect = rect;
            }
            ui.painter()
                .rect_filled(rect, 4.0, egui::Color32::from_rgb(24, 28, 35));
            if !self.gizmo.active() && response.dragged_by(egui::PointerButton::Primary) {
                let delta = response.drag_motion();
                self.yaw += delta.x * 0.01;
                self.pitch = (self.pitch + delta.y * 0.01).clamp(-1.55, 1.55);
            }
            if !self.gizmo.active() && response.dragged_by(egui::PointerButton::Secondary) {
                let delta = response.drag_motion();
                self.pan +=
                    egui::vec2(delta.x * 2.0 / rect.width(), -delta.y * 2.0 / rect.height());
            }
            if !self.gizmo.active() && response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                self.zoom = (self.zoom * (scroll * 0.002).exp()).clamp(0.1, 8.0);
            }
            if self.interactive && !self.gizmo.active() && response.clicked() {
                if let (Some(geometry), Some(pos)) =
                    (&self.geometry, response.interact_pointer_pos())
                {
                    let visible: Vec<_> = self.parts.iter().map(|p| p.visible).collect();
                    if let Some(section) = picking::pick_wires(
                        &self.collision_wires,
                        View {
                            yaw: self.yaw,
                            pitch: self.pitch,
                            zoom: self.zoom,
                            pan: self.pan,
                        },
                        rect,
                        pos,
                    )
                    .filter(|_| self.show_collisions)
                    .or_else(|| {
                        picking::pick(
                            geometry,
                            &visible,
                            false,
                            View {
                                yaw: self.yaw,
                                pitch: self.pitch,
                                zoom: self.zoom,
                                pan: self.pan,
                            },
                            rect,
                            pos,
                        )
                    }) {
                        self.selected = section;
                        self.selection_changed = Some(section);
                    }
                }
            }
            let view = View {
                yaw: self.yaw,
                pitch: self.pitch,
                zoom: self.zoom,
                pan: self.pan,
            };
            if let Some(mesh) = &self.mesh {
                let mesh = mesh.clone();
                let visible: Vec<_> = self.parts.iter().map(|p| p.visible).collect();
                let textures = self.use_textures;
                let collisions = false;
                let selected = self.selected;
                let aspect = rect.width() / rect.height();
                let callback = egui_glow::CallbackFn::new(move |_info, painter| {
                    mesh.paint(
                        painter.gl(),
                        view,
                        aspect,
                        &visible,
                        DrawOptions {
                            use_textures: textures,
                            show_collisions: collisions,
                            selected,
                        },
                    );
                });
                ui.painter().add(egui::PaintCallback {
                    rect,
                    callback: Arc::new(callback),
                });
            }
            if let Some(geometry) = self.geometry.as_ref().filter(|_| self.interactive) {
                if let Some(edit) =
                    self.gizmo
                        .draw(ui, geometry, &self.document, self.selected, view, rect)
                {
                    self.edits.push(edit);
                }
            }
            if self.show_collisions {
                if let Some(geometry) = &self.geometry {
                    let mut draft = self.gizmo.preview_document(&self.document);
                    if let Some(edit) = self.edits.last() {
                        let draft = draft.get_or_insert_with(|| self.document.clone());
                        if let Some(index) = draft
                            .properties()
                            .iter()
                            .position(|p| p.section == Some(edit.section) && p.key == edit.key)
                        {
                            let _ = draft.set_value(index, &edit.value);
                        } else {
                            let _ = draft.add_property(Some(edit.section), &edit.key, &edit.value);
                        }
                    }
                    let document = draft.as_ref().unwrap_or(&self.document);
                    if self.collision_source != document.source() {
                        self.collision_wires = geometry.preview_collisions(document);
                        self.collision_source = document.source().into();
                    }
                    for wire in &self.collision_wires {
                        let color = if self.selected == Some(wire.section) {
                            egui::Color32::from_rgb(255, 166, 26)
                        } else {
                            egui::Color32::from_rgb(64, 255, 89)
                        };
                        for edge in wire.vertices.as_chunks::<2>().0 {
                            ui.painter().with_clip_rect(rect).line_segment(
                                [
                                    picking::project(edge[0], view, rect).0,
                                    picking::project(edge[1], view, rect).0,
                                ],
                                egui::Stroke::new(1.5, color),
                            );
                        }
                    }
                }
            }
        });
    }
}

struct PartVisibility {
    section: Option<usize>,
    name: String,
    visible: bool,
    details: String,
}
#[derive(Clone, Copy)]
struct View {
    yaw: f32,
    pitch: f32,
    zoom: f32,
    pan: egui::Vec2,
}
#[derive(Clone, Copy)]
struct DrawOptions {
    use_textures: bool,
    show_collisions: bool,
    selected: Option<usize>,
}
struct DrawBatch {
    section: Option<usize>,
    part: usize,
    first: i32,
    count: i32,
    color: [f32; 4],
    texture: Option<usize>,
    transparent: bool,
    center: [f32; 3],
}
struct GpuMesh {
    gl: Arc<glow::Context>,
    program: glow::Program,
    vertex_array: glow::VertexArray,
    buffer: glow::Buffer,
    batches: Vec<DrawBatch>,
    textures: Vec<glow::Texture>,
    collision_batches: Vec<DrawBatch>,
}
impl GpuMesh {
    fn new(gl: Arc<glow::Context>, mesh: &ObjMesh) -> Result<Self, String> {
        let version = egui_glow::ShaderVersion::get(&gl);
        if !version.is_new_shader_interface() {
            return Err("L’aperçu nécessite OpenGL 3.1 ou OpenGL ES 3".into());
        }
        // Les ressources OpenGL sont créées, dessinées et détruites sur le thread UI,
        // avec le contexte eframe actif. Le worker ne manipule que la géométrie CPU.
        unsafe {
            let program = gl.create_program()?;
            let mut shaders = Vec::new();
            let result = (|| -> Result<(), String> {
                for (kind, source) in [
                    (glow::VERTEX_SHADER, VERTEX_SHADER),
                    (glow::FRAGMENT_SHADER, FRAGMENT_SHADER),
                ] {
                    let shader = gl.create_shader(kind)?;
                    shaders.push(shader);
                    gl.shader_source(
                        shader,
                        &format!("{}\n{source}", version.version_declaration()),
                    );
                    gl.compile_shader(shader);
                    if !gl.get_shader_compile_status(shader) {
                        return Err(gl.get_shader_info_log(shader));
                    }
                    gl.attach_shader(program, shader);
                }
                gl.bind_attrib_location(program, 0, "a_position");
                gl.bind_attrib_location(program, 1, "a_normal");
                gl.bind_attrib_location(program, 2, "a_uv");
                gl.link_program(program);
                if !gl.get_program_link_status(program) {
                    return Err(gl.get_program_info_log(program));
                }
                Ok(())
            })();
            for shader in shaders {
                gl.delete_shader(shader);
            }
            if let Err(error) = result {
                gl.delete_program(program);
                return Err(format!("Création de l’aperçu : {error}"));
            }
            let vertex_array = match gl.create_vertex_array() {
                Ok(value) => value,
                Err(error) => {
                    gl.delete_program(program);
                    return Err(error);
                }
            };
            let buffer = match gl.create_buffer() {
                Ok(value) => value,
                Err(error) => {
                    gl.delete_vertex_array(vertex_array);
                    gl.delete_program(program);
                    return Err(error);
                }
            };
            let mut gpu = Self {
                gl: gl.clone(),
                program,
                vertex_array,
                buffer,
                batches: Vec::new(),
                textures: Vec::new(),
                collision_batches: Vec::new(),
            };
            let mut bytes: Vec<u8> = mesh
                .vertices
                .iter()
                .flatten()
                .flat_map(|v| v.to_ne_bytes())
                .collect();
            let mut first = mesh.vertices.len();
            for wire in &mesh.collisions {
                for p in &wire.vertices {
                    for v in [p[0], p[1], p[2], 0.0, 0.0, 1.0, 0.0, 0.0] {
                        bytes.extend(v.to_ne_bytes());
                    }
                }
                gpu.collision_batches.push(DrawBatch {
                    section: Some(wire.section),
                    part: 0,
                    first: first as i32,
                    count: wire.vertices.len() as i32,
                    color: [0.25, 1.0, 0.35, 1.0],
                    texture: None,
                    transparent: false,
                    center: [0.0; 3],
                });
                first += wire.vertices.len();
            }
            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 32, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 32, 12);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 32, 24);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.active_texture(glow::TEXTURE0);
            let max_size = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as u32;
            for image in &mesh.textures {
                if image.width > max_size || image.height > max_size {
                    return Err(format!(
                        "Texture trop grande pour la carte graphique : {}",
                        image.path.display()
                    ));
                }
                let texture = gl.create_texture()?;
                gpu.textures.push(texture);
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::LINEAR as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::LINEAR as i32,
                );
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA8 as i32,
                    image.width as i32,
                    image.height as i32,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(&image.rgba)),
                );
            }
            gl.bind_texture(glow::TEXTURE_2D, None);
            for (part_index, part) in mesh.parts.iter().enumerate() {
                for batch in &part.batches {
                    let material = batch.material.and_then(|i| mesh.materials.get(i));
                    let color = material
                        .map(|m| m.diffuse)
                        .unwrap_or([0.56, 0.68, 0.82, 1.0]);
                    let texture = material.and_then(|m| m.texture).filter(|_| batch.has_uv);
                    let transparent =
                        color[3] < 0.999 || texture.is_some_and(|i| mesh.textures[i].has_alpha);
                    let mut center = [0.0; 3];
                    for v in &mesh.vertices[batch.first..batch.first + batch.count] {
                        for axis in 0..3 {
                            center[axis] += v[axis] / batch.count as f32;
                        }
                    }
                    gpu.batches.push(DrawBatch {
                        section: part.section,
                        part: part_index,
                        first: batch.first as i32,
                        count: batch.count as i32,
                        color,
                        texture,
                        transparent,
                        center,
                    });
                }
            }
            Ok(gpu)
        }
    }
    fn paint(
        &self,
        gl: &glow::Context,
        view: View,
        aspect: f32,
        visible: &[bool],
        options: DrawOptions,
    ) {
        let DrawOptions {
            use_textures,
            show_collisions,
            selected,
        } = options;
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.depth_mask(true);
            gl.clear_depth_f32(1.0);
            gl.clear(glow::DEPTH_BUFFER_BIT);
            gl.disable(glow::CULL_FACE);
            gl.disable(glow::BLEND);
            gl.use_program(Some(self.program));
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_angles").as_ref(),
                view.yaw,
                view.pitch,
            );
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_scale").as_ref(),
                view.zoom / aspect.max(1.0),
                view.zoom * aspect.min(1.0),
            );
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_pan").as_ref(),
                view.pan.x,
                view.pan.y,
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "u_texture").as_ref(),
                0,
            );
            gl.uniform_1_i32(gl.get_uniform_location(self.program, "u_unlit").as_ref(), 0);
            gl.active_texture(glow::TEXTURE0);
            gl.bind_vertex_array(Some(self.vertex_array));
            let mut transparent = Vec::new();
            for batch in &self.batches {
                if !visible.get(batch.part).copied().unwrap_or(false) {
                    continue;
                }
                if batch.color[3] < 0.999 || (use_textures && batch.transparent) {
                    transparent.push(batch);
                } else {
                    self.draw(
                        gl,
                        batch,
                        use_textures,
                        selected.is_some() && batch.section == selected,
                    );
                }
            }
            transparent.sort_by(|a, b| depth(a.center, view).total_cmp(&depth(b.center, view)));
            gl.enable(glow::BLEND);
            gl.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );
            gl.depth_mask(false);
            for batch in transparent {
                self.draw(
                    gl,
                    batch,
                    use_textures,
                    selected.is_some() && batch.section == selected,
                );
            }
            if show_collisions {
                gl.disable(glow::DEPTH_TEST);
                gl.disable(glow::BLEND);
                gl.uniform_1_i32(gl.get_uniform_location(self.program, "u_unlit").as_ref(), 1);
                gl.uniform_1_i32(
                    gl.get_uniform_location(self.program, "u_textured").as_ref(),
                    0,
                );
                gl.uniform_4_f32(
                    gl.get_uniform_location(self.program, "u_color").as_ref(),
                    0.25,
                    1.0,
                    0.35,
                    1.0,
                );
                gl.line_width(1.0);
                for batch in &self.collision_batches {
                    let c = if selected.is_some() && batch.section == selected {
                        [1.0, 0.65, 0.1]
                    } else {
                        [0.25, 1.0, 0.35]
                    };
                    gl.uniform_4_f32(
                        gl.get_uniform_location(self.program, "u_color").as_ref(),
                        c[0],
                        c[1],
                        c[2],
                        1.0,
                    );
                    gl.draw_arrays(glow::LINES, batch.first, batch.count);
                }
            }
            gl.depth_mask(true);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::DEPTH_TEST);
        }
    }
    fn draw(&self, gl: &glow::Context, batch: &DrawBatch, use_textures: bool, selected: bool) {
        unsafe {
            let texture = batch
                .texture
                .filter(|_| use_textures)
                .map(|i| self.textures[i]);
            gl.bind_texture(glow::TEXTURE_2D, texture);
            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "u_textured").as_ref(),
                i32::from(texture.is_some()),
            );
            gl.uniform_4_f32(
                gl.get_uniform_location(self.program, "u_color").as_ref(),
                if selected { 1.0 } else { batch.color[0] },
                if selected { 0.65 } else { batch.color[1] },
                if selected { 0.1 } else { batch.color[2] },
                batch.color[3],
            );
            gl.draw_arrays(glow::TRIANGLES, batch.first, batch.count);
        }
    }
}
fn depth(p: [f32; 3], view: View) -> f32 {
    let z = -view.yaw.sin() * p[0] + view.yaw.cos() * p[2];
    view.pitch.sin() * p[1] + view.pitch.cos() * z
}

impl Drop for GpuMesh {
    fn drop(&mut self) {
        unsafe {
            for texture in &self.textures {
                self.gl.delete_texture(*texture);
            }
            self.gl.delete_buffer(self.buffer);
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_program(self.program);
        }
    }
}
const VERTEX_SHADER: &str = r#"
precision highp float;
in vec3 a_position;
in vec3 a_normal;
in vec2 a_uv;
out vec2 v_uv;
uniform vec2 u_angles;
uniform vec2 u_scale;
uniform vec2 u_pan;
out vec3 v_normal;
vec3 rotate(vec3 p) {
    float c=cos(u_angles.x), s=sin(u_angles.x);
    p=vec3(c*p.x+s*p.z,p.y,-s*p.x+c*p.z);
    c=cos(u_angles.y);s=sin(u_angles.y);
    return vec3(p.x,c*p.y-s*p.z,s*p.y+c*p.z);
}
void main() {
    vec3 p=rotate(a_position);
    gl_Position=vec4(p.xy*u_scale+u_pan,-p.z*0.5,1.0);
    v_normal=rotate(a_normal);
    v_uv=a_uv;
}
"#;
const FRAGMENT_SHADER: &str = r#"
precision mediump float;
in vec3 v_normal;
in vec2 v_uv;
uniform sampler2D u_texture;
uniform bool u_textured;
uniform bool u_unlit;
uniform vec4 u_color;
out vec4 out_color;
void main() {
    vec3 n=normalize(v_normal)*(gl_FrontFacing?1.0:-1.0);
    float light=0.3+0.7*max(0.0,dot(n,normalize(vec3(-0.4,0.7,1.0))));
    vec4 color=u_color;
    if(u_textured){color*=texture(u_texture,v_uv);}
    if(color.a<0.01){discard;}
    out_color=vec4(color.rgb*(u_unlit?1.0:light),color.a);
}
"#;

impl Drop for Preview {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
struct LoadTask {
    request: ObjRequest,
    sender: mpsc::Sender<Result<ObjMesh, String>>,
    cancel: Arc<AtomicBool>,
    ctx: egui::Context,
}
type Pending = Arc<(Mutex<Option<LoadTask>>, Condvar)>;
fn background(
    request: ObjRequest,
    ctx: &egui::Context,
) -> (mpsc::Receiver<Result<ObjMesh, String>>, Arc<AtomicBool>) {
    static LOADER: OnceLock<Pending> = OnceLock::new();
    let pending = LOADER.get_or_init(|| {
        let shared: Pending = Arc::new((Mutex::new(None), Condvar::new()));
        let worker = shared.clone();
        std::thread::spawn(move || loop {
            let task = {
                let (lock, ready) = &*worker;
                let mut pending = lock.lock().unwrap_or_else(|e| e.into_inner());
                while pending.is_none() {
                    pending = ready.wait(pending).unwrap_or_else(|e| e.into_inner());
                }
                pending.take().unwrap()
            };
            if task.cancel.load(Ordering::Relaxed) {
                continue;
            }
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| task.request.load()))
                    .unwrap_or_else(|_| Err("Chargement du modèle interrompu".into()));
            if !task.cancel.load(Ordering::Relaxed) {
                let _ = task.sender.send(result);
                task.ctx.request_repaint();
            }
        });
        shared
    });
    let (sender, receiver) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let (lock, ready) = &**pending;
    *lock.lock().unwrap_or_else(|e| e.into_inner()) = Some(LoadTask {
        request,
        sender,
        cancel: cancel.clone(),
        ctx: ctx.clone(),
    });
    ready.notify_one();
    (receiver, cancel)
}

#[cfg(test)]
mod window_tests {
    use super::*;
    fn fixture(ctx: &egui::Context) -> Preview {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("pack_info.dynx"), "PackName: Test\n").unwrap();
        std::fs::write(
            root.path().join("car.obj"),
            "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
        )
        .unwrap();
        let pack = cpm_core::Pack::open(root.path()).unwrap();
        let doc = cpm_core::Document::parse("Model: car.obj\n").unwrap();
        let request =
            ObjRequest::resolve(Some(&pack), std::path::Path::new("vehicle_test.dynx"), &doc)
                .unwrap();
        let mut preview = Preview::open(request, ctx);
        preview.cancel.store(true, Ordering::Relaxed);
        preview.receiver = None;
        preview.parts = (0..100)
            .map(|i| PartVisibility {
                section: Some(i),
                name: format!("Piece_{i}"),
                visible: true,
                details: String::new(),
            })
            .collect();
        preview
    }
    fn frame(
        preview: &mut Preview,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> Vec<egui::epaint::ClippedShape> {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500.0, 1000.0),
                )),
                events,
                ..Default::default()
            },
            |_| {
                assert!(preview.show(ctx, None));
            },
        );
        output.textures_delta.clear();
        output.shapes
    }
    fn find(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Pos2> {
        match shape {
            egui::epaint::Shape::Text(t) if t.galley.job.text == label => {
                Some(t.pos + t.galley.rect.center().to_vec2())
            }
            egui::epaint::Shape::Vec(v) => v.iter().find_map(|s| find(s, label)),
            _ => None,
        }
    }
    #[test]
    fn reattachment_preserves_camera_visibility_and_full_height_canvas() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx);
        let mut preview = fixture(&ctx);
        assert!(preview.detached);
        preview.yaw = 1.1;
        preview.zoom = 1.7;
        preview.live = false;
        preview.parts[0].visible = false;
        frame(&mut preview, &ctx, vec![]);
        let shapes = frame(&mut preview, &ctx, vec![]);
        let options = shapes
            .iter()
            .find_map(|s| find(&s.shape, "Actualisation automatique"))
            .unwrap();
        assert!(preview.scene_rect.left() > options.x);
        assert!(preview.scene_rect.height() > 300.0);
        let pos = shapes
            .iter()
            .find_map(|s| find(&s.shape, "Intégrer à CPM"))
            .unwrap();
        frame(&mut preview, &ctx, vec![egui::Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            frame(
                &mut preview,
                &ctx,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
        assert!(!preview.detached);
        frame(&mut preview, &ctx, vec![]);
        let shapes = frame(&mut preview, &ctx, vec![]);
        assert!(shapes
            .iter()
            .any(|s| find(&s.shape, "Détacher la fenêtre…").is_some()));
        assert_eq!(preview.yaw, 1.1);
        assert_eq!(preview.zoom, 1.7);
        assert!(!preview.live);
        assert!(!preview.parts[0].visible);
        assert!(preview.scene_rect.height() > 300.0);
    }
    fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    #[test]
    fn move_and_resize_allow_camera_selection_and_exclusive_handle_drag() {
        for tool in [gizmo::Tool::Move, gizmo::Tool::Resize] {
            let ctx = egui::Context::default();
            crate::theme::apply(&ctx);
            let mut preview = fixture(&ctx);
            preview.parts.clear();
            preview.geometry = Some(Arc::new(
                ObjMesh::parse(b"v -10 -10 0\nv 10 -10 0\nv 0 10 0\nf 1 2 3\n").unwrap(),
            ));
            preview.document = cpm_core::Document::parse("Shape_a{\nPosition: 0 0 0\nScale: 1 1 1\n}\nShape_b{\nPosition: 4 2 0\nScale: 1 1 1\n}\n").unwrap();
            let original = preview.document.source().to_owned();
            preview.selected = Some(0);
            preview.show_collisions = true;
            preview.gizmo.tool = tool;
            frame(&mut preview, &ctx, vec![]);
            frame(&mut preview, &ctx, vec![]);
            let start = preview.scene_rect.left_top() + egui::vec2(25.0, 25.0);
            let yaw = preview.yaw;
            frame(&mut preview, &ctx, vec![egui::Event::PointerMoved(start)]);
            frame(&mut preview, &ctx, vec![pointer(start, true)]);
            frame(
                &mut preview,
                &ctx,
                vec![egui::Event::PointerMoved(start + egui::vec2(45.0, 20.0))],
            );
            assert_ne!(
                preview.yaw, yaw,
                "orbit must remain available in both edit tools"
            );
            frame(
                &mut preview,
                &ctx,
                vec![pointer(start + egui::vec2(45.0, 20.0), false)],
            );
            assert!(preview.take_edits().is_empty());
            let view = View {
                yaw: preview.yaw,
                pitch: preview.pitch,
                zoom: preview.zoom,
                pan: preview.pan,
            };
            let point = preview.collision_wires[1]
                .vertices
                .iter()
                .map(|p| picking::project(*p, view, preview.scene_rect).0)
                .find(|p| {
                    preview.scene_rect.contains(*p)
                        && picking::pick_wires(
                            &preview.collision_wires,
                            view,
                            preview.scene_rect,
                            *p,
                        ) == Some(Some(1))
                })
                .unwrap();
            frame(&mut preview, &ctx, vec![egui::Event::PointerMoved(point)]);
            frame(&mut preview, &ctx, vec![pointer(point, true)]);
            frame(&mut preview, &ctx, vec![pointer(point, false)]);
            assert_eq!(
                preview.selected,
                Some(1),
                "click selects another box without changing tools"
            );
            frame(&mut preview, &ctx, vec![]);
            let mesh = preview.geometry.as_ref().unwrap();
            let origin = picking::project(
                mesh.normalize_point(gizmo::config_world([4.0, 2.0, 0.0], [1.0; 3])),
                view,
                preview.scene_rect,
            )
            .0;
            let unit = picking::project(
                mesh.normalize_point(gizmo::config_world([5.0, 2.0, 0.0], [1.0; 3])),
                view,
                preview.scene_rect,
            )
            .0 - origin;
            let endpoint = origin + unit.normalized() * 75.0;
            let yaw = preview.yaw;
            frame(
                &mut preview,
                &ctx,
                vec![egui::Event::PointerMoved(endpoint)],
            );
            frame(&mut preview, &ctx, vec![pointer(endpoint, true)]);
            frame(
                &mut preview,
                &ctx,
                vec![egui::Event::PointerMoved(
                    endpoint + unit.normalized() * 25.0,
                )],
            );
            assert!(preview.manipulating());
            assert_eq!(preview.yaw, yaw, "handle drag must not rotate the camera");
            frame(
                &mut preview,
                &ctx,
                vec![pointer(endpoint + unit.normalized() * 25.0, false)],
            );
            let edits = preview.take_edits();
            assert_eq!(edits.len(), 1);
            assert_eq!(edits[0].section, 1);
            assert_eq!(preview.document.source(), original);
            assert!(preview.gizmo.tool == tool);
        }
    }
}
