use cpm_core::{ObjMesh, ObjRequest};
use eframe::{egui, egui_glow, glow};
use glow::HasContext as _;
use std::sync::{mpsc, Arc};

pub struct Preview {
    path: String,
    receiver: Option<mpsc::Receiver<Result<ObjMesh, String>>>,
    mesh: Option<Arc<GpuMesh>>,
    error: Option<String>,
    statistics: String,
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
            path,
            receiver: Some(receiver),
            mesh: None,
            error: None,
            statistics: String::new(),
            yaw: -0.6,
            pitch: 0.25,
            zoom: 0.9,
            pan: egui::Vec2::ZERO,
        }
    }
    pub fn show(&mut self, ctx: &egui::Context, gl: Option<&Arc<glow::Context>>) -> bool {
        if let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    self.receiver = None;
                    let result = result.and_then(|mesh| {
                        self.statistics =
                            format!("{} triangles · {} objets", mesh.triangles, mesh.objects);
                        let gl = gl.ok_or("Le moteur OpenGL de l’aperçu n’est pas disponible")?;
                        GpuMesh::new(gl.clone(), &mesh)
                    });
                    match result {
                        Ok(mesh) => self.mesh = Some(Arc::new(mesh)),
                        Err(error) => self.error = Some(error),
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
            ui.weak("Géométrie du modèle uniquement — sans textures, animations ni transformations DynamX.");
            if self.receiver.is_some() { ui.spinner(); ui.label("Chargement du modèle…"); }
            if let Some(error) = &self.error { ui.colored_label(egui::Color32::LIGHT_RED,error); }
            let size=egui::vec2(ui.available_width().max(1.0), ui.available_height().max(180.0));
            let (rect,response)=ui.allocate_exact_size(size,egui::Sense::drag());
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
            if let Some(mesh) = &self.mesh {
                let mesh=mesh.clone(); let yaw=self.yaw; let pitch=self.pitch; let zoom=self.zoom; let pan=self.pan;
                let aspect=rect.width()/rect.height();
                let callback=egui_glow::CallbackFn::new(move |_info,painter| { mesh.paint(painter.gl(),yaw,pitch,zoom,aspect,pan); });
                ui.painter().add(egui::PaintCallback {rect,callback:Arc::new(callback)});
            }
        });
        open
    }
}

struct GpuMesh {
    gl: Arc<glow::Context>,
    program: glow::Program,
    vertex_array: glow::VertexArray,
    buffer: glow::Buffer,
    count: i32,
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
            let gpu = Self {
                gl: gl.clone(),
                program,
                vertex_array,
                buffer,
                count: mesh.vertices.len() as i32,
            };
            let bytes: Vec<u8> = mesh
                .vertices
                .iter()
                .flatten()
                .flat_map(|v| v.to_ne_bytes())
                .collect();
            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 24, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 24, 12);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            Ok(gpu)
        }
    }
    fn paint(
        &self,
        gl: &glow::Context,
        yaw: f32,
        pitch: f32,
        zoom: f32,
        aspect: f32,
        pan: egui::Vec2,
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
                yaw,
                pitch,
            );
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_scale").as_ref(),
                zoom / aspect.max(1.0),
                zoom * aspect.min(1.0),
            );
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_pan").as_ref(),
                pan.x,
                pan.y,
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, self.count);
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
        }
    }
}
impl Drop for GpuMesh {
    fn drop(&mut self) {
        unsafe {
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
}
"#;
const FRAGMENT_SHADER: &str = r#"
precision mediump float;
in vec3 v_normal;
out vec4 out_color;
void main() {
    vec3 n=normalize(v_normal)*(gl_FrontFacing?1.0:-1.0);
    float light=0.3+0.7*max(0.0,dot(n,normalize(vec3(-0.4,0.7,1.0))));
    out_color=vec4(vec3(0.56,0.68,0.82)*light,1.0);
}
"#;
