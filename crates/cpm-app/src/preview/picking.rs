use super::View;
use cpm_core::ObjMesh;
use eframe::egui::{Pos2, Rect};
pub(super) fn project(p: [f32; 3], view: View, rect: Rect) -> (Pos2, f32) {
    let x = view.yaw.cos() * p[0] + view.yaw.sin() * p[2];
    let z = -view.yaw.sin() * p[0] + view.yaw.cos() * p[2];
    let y = view.pitch.cos() * p[1] - view.pitch.sin() * z;
    let depth = view.pitch.sin() * p[1] + view.pitch.cos() * z;
    let aspect = rect.width() / rect.height();
    (
        Pos2::new(
            rect.center().x + (x * view.zoom / aspect.max(1.0) + view.pan.x) * rect.width() * 0.5,
            rect.center().y - (y * view.zoom * aspect.min(1.0) + view.pan.y) * rect.height() * 0.5,
        ),
        depth,
    )
}
fn triangle(point: Pos2, a: Pos2, b: Pos2, c: Pos2) -> Option<[f32; 3]> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = point - a;
    let det = v0.x * v1.y - v1.x * v0.y;
    if det.abs() < 0.00001 {
        return None;
    }
    let u = (v2.x * v1.y - v1.x * v2.y) / det;
    let v = (v0.x * v2.y - v2.x * v0.y) / det;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0).then_some([1.0 - u - v, u, v])
}
pub(super) fn segment_distance(point: Pos2, a: Pos2, b: Pos2) -> f32 {
    let edge = b - a;
    let length = edge.length_sq();
    if length < 0.0001 {
        return point.distance(a);
    }
    point.distance(a + edge * ((point - a).dot(edge) / length).clamp(0.0, 1.0))
}
pub(super) fn pick(
    mesh: &ObjMesh,
    visible: &[bool],
    collisions: bool,
    view: View,
    rect: Rect,
    point: Pos2,
) -> Option<Option<usize>> {
    if collisions {
        let mut closest = 6.0;
        let mut section = None;
        for wire in &mesh.collisions {
            for edge in wire.vertices.as_chunks::<2>().0 {
                let distance = segment_distance(
                    point,
                    project(edge[0], view, rect).0,
                    project(edge[1], view, rect).0,
                );
                if distance < closest {
                    closest = distance;
                    section = Some(Some(wire.section));
                }
            }
        }
        if section.is_some() {
            return section;
        }
    }
    let mut closest = f32::NEG_INFINITY;
    let mut section = None;
    for (index, part) in mesh
        .parts
        .iter()
        .enumerate()
        .filter(|(i, _)| visible.get(*i) == Some(&true))
    {
        let _ = index;
        for batch in &part.batches {
            for vertices in mesh.vertices[batch.first..batch.first + batch.count]
                .as_chunks::<3>()
                .0
            {
                let projected: Vec<_> = vertices
                    .iter()
                    .map(|v| project([v[0], v[1], v[2]], view, rect))
                    .collect();
                if let Some(weights) =
                    triangle(point, projected[0].0, projected[1].0, projected[2].0)
                {
                    let depth = (0..3).map(|i| weights[i] * projected[i].1).sum();
                    if depth > closest {
                        closest = depth;
                        section = Some(part.section);
                    }
                }
            }
        }
    }
    section
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picks_frontmost_visible_part_and_wire_edges() {
        let mut mesh=ObjMesh::parse(b"o back\nv -1 -1 0\nv 1 -1 0\nv 0 1 0\nf 1 2 3\no front\nv -1 -1 1\nv 1 -1 1\nv 0 1 1\nf 4 5 6\n").unwrap();
        mesh.parts[0].section = Some(1);
        mesh.parts[1].section = Some(2);
        let view = View {
            yaw: 0.0,
            pitch: 0.0,
            zoom: 1.0,
            pan: eframe::egui::Vec2::ZERO,
        };
        let rect = Rect::from_min_size(Pos2::ZERO, eframe::egui::vec2(400.0, 400.0));
        assert_eq!(
            pick(&mesh, &[true, true], false, view, rect, rect.center()),
            Some(Some(2))
        );
        assert_eq!(
            pick(&mesh, &[true, false], false, view, rect, rect.center()),
            Some(Some(1))
        );
        assert_eq!(
            pick(&mesh, &[false, false], false, view, rect, rect.center()),
            None
        );
        assert!(
            (segment_distance(Pos2::new(2.0, 1.0), Pos2::ZERO, Pos2::new(4.0, 0.0)) - 1.0).abs()
                < 0.001
        );
    }
}
