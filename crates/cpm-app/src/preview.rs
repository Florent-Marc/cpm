mod picking;
use cpm_core::{ObjMesh, ObjRequest};
use eframe::{egui, egui_glow, glow};
use glow::HasContext as _;
use std::sync::{mpsc, Arc};

pub struct Preview {
    pub live: bool,
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
        let path = request.path.display().to_string();
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = sender.send(request.load());
            ctx.request_repaint();
        });
        Self {
            live: true,
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
    pub fn take_selection(&mut self) -> Option<Option<usize>> {
        self.selection_changed.take()
    }
    pub fn busy(&self) -> bool {
        self.receiver.is_some()
    }
    pub fn refresh(&mut self, request: ObjRequest, ctx: &egui::Context) {
        self.path = request.path.display().to_string();
        self.error = None;
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(request.load());
            ctx.request_repaint();
        });
    }
    pub fn fail(&mut self, error: String) {
        self.error = Some(error);
        self.mesh = None;
        self.geometry = None;
    }
    pub fn show(&mut self, ctx: &egui::Context, gl: Option<&Arc<glow::Context>>) -> bool {
        if let Some(receiver) = &self.receiver {
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
        egui::Window::new("Aperçu 3D — OBJ").open(&mut open).default_size([760.0,560.0]).min_size([360.0,300.0]).resizable(true).collapsible(false).show(ctx, |ui| {
            ui.label(&self.path);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Recentrer").clicked() { self.yaw = -0.6; self.pitch=0.25; self.zoom=0.9; self.pan=egui::Vec2::ZERO; }
                if ui.button("Face").clicked() { self.yaw=0.0; self.pitch=0.0; }
                if ui.button("Côté").clicked() { self.yaw=std::f32::consts::FRAC_PI_2; self.pitch=0.0; }
                if ui.button("Dessus").clicked() { self.yaw=0.0; self.pitch=std::f32::consts::FRAC_PI_2; }
                ui.label(&self.statistics);
            });
            ui.label("Glisser : tourner · Molette : zoomer · Glisser avec le bouton droit : déplacer");
            ui.weak("Matériaux, roues et volant — aperçu statique, sans variantes DynamX. Actualisation après une courte pause de saisie.");
            ui.checkbox(&mut self.live,"Actualisation automatique");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.use_textures,"Afficher les textures");
                ui.checkbox(&mut self.show_collisions,format!("Afficher les collisions ({})",self.collision_names.len())).on_hover_text(format!("Boîtes Shape… en fil de fer, visibles à travers le modèle. Les collisions générées automatiquement ne sont pas affichées.\n{}",self.collision_names.join("\n")));
            });
            if !self.parts.is_empty() {
                egui::CollapsingHeader::new(format!("Pièces ({})",self.parts.len())).default_open(true).show(ui,|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Tout afficher").clicked(){for p in &mut self.parts{p.visible=true;}}
                        if ui.button("Tout masquer").clicked(){for p in &mut self.parts{p.visible=false;}}
                    });
                    let mut isolate=None;
                    egui::ScrollArea::vertical().id_salt("obj-parts").max_height(125.0).show(ui,|ui| {
                        for (index,part) in self.parts.iter_mut().enumerate(){
                            ui.push_id(index,|ui|{ui.horizontal(|ui|{
                                ui.checkbox(&mut part.visible,&part.name).on_hover_text(&part.details);
                                if ui.small_button("Isoler").clicked(){isolate=Some(index);}
                            });});
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
            let size=egui::vec2(ui.available_width().max(1.0), ui.available_height().max(180.0));
            let (rect,response)=ui.allocate_exact_size(size,egui::Sense::click_and_drag());
            ui.painter().rect_filled(rect,4.0,egui::Color32::from_rgb(24,28,35));
            if response.dragged_by(egui::PointerButton::Primary) {
                let delta=response.drag_motion(); self.yaw+=delta.x*0.01; self.pitch=(self.pitch+delta.y*0.01).clamp(-1.55,1.55);
            }
            if response.dragged_by(egui::PointerButton::Secondary) {
                let delta=response.drag_motion(); self.pan+=egui::vec2(delta.x*2.0/rect.width(),-delta.y*2.0/rect.height());
            }
            if response.hovered() {
                let scroll=ui.input(|i|i.smooth_scroll_delta.y);
                self.zoom=(self.zoom*(scroll*0.002).exp()).clamp(0.1,8.0);
            }
            if response.clicked() {
                if let (Some(geometry),Some(pos))=(&self.geometry,response.interact_pointer_pos()) {
                    let visible:Vec<_>=self.parts.iter().map(|p|p.visible).collect();
                    if let Some(section)=picking::pick(geometry,&visible,self.show_collisions,View{yaw:self.yaw,pitch:self.pitch,zoom:self.zoom,pan:self.pan},rect,pos) {
                        self.selected=section;self.selection_changed=Some(section);
                    }
                }
            }
            if let Some(mesh) = &self.mesh {
                let mesh=mesh.clone(); let view=View {yaw:self.yaw,pitch:self.pitch,zoom:self.zoom,pan:self.pan};
                let visible:Vec<_>=self.parts.iter().map(|p|p.visible).collect();let textures=self.use_textures;let collisions=self.show_collisions;let selected=self.selected;
                let aspect=rect.width()/rect.height();
                let callback=egui_glow::CallbackFn::new(move |_info,painter| { mesh.paint(painter.gl(),view,aspect,&visible,textures,collisions,selected); });
                ui.painter().add(egui::PaintCallback {rect,callback:Arc::new(callback)});
            }
        });
        open
    }
}

struct PartVisibility {
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
        use_textures: bool,
        show_collisions: bool,
        selected: Option<usize>,
    ) {
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
