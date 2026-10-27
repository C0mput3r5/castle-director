use crate::blueprint::TemporaryProp;
use crate::camera::{ExteriorShell, TerrainPlinth};
use crate::npc::ActivityFeed;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use std::fs::File;
use std::io::{BufWriter, Write};

pub const STL_OUTPUT_PATH: &str = "exported_castle_diorama.stl";

pub struct StlExportPlugin;

impl Plugin for StlExportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StlExportStatus>()
            .add_systems(Update, (handle_stl_export_hotkey, tick_stl_banner_timer));
    }
}

#[derive(Resource, Default)]
pub struct StlExportStatus {
    pub last_banner_message: String,
    pub banner_timer: f32,
    pub trigger_requested: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct StlTriangle {
    pub normal: Vec3,
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
}

impl StlTriangle {
    pub fn from_vertices(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        let normal = (v1 - v0).cross(v2 - v0).normalize_or_zero();
        Self { normal, v0, v1, v2 }
    }
}

pub fn serialize_binary_stl(triangles: &[StlTriangle]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(84 + triangles.len() * 50);

    // 1. 80-byte ASCII header
    let header_str = b"Castle Director - Watertight 3D Printable Diorama (ExteriorShell + Plinth)";
    let mut header = [0u8; 80];
    let copy_len = header_str.len().min(80);
    header[..copy_len].copy_from_slice(&header_str[..copy_len]);
    bytes.extend_from_slice(&header);

    // 2. 4-byte little-endian u32 triangle count
    let count = triangles.len() as u32;
    bytes.extend_from_slice(&count.to_le_bytes());

    // 3. 50-byte records per triangle: normal (12B) + v0 (12B) + v1 (12B) + v2 (12B) + attr (2B)
    for tri in triangles {
        for v in [tri.normal, tri.v0, tri.v1, tri.v2] {
            bytes.extend_from_slice(&v.x.to_le_bytes());
            bytes.extend_from_slice(&v.y.to_le_bytes());
            bytes.extend_from_slice(&v.z.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
    }

    bytes
}

pub fn extract_world_triangles_from_mesh(mesh: &Mesh, transform: &GlobalTransform) -> Vec<StlTriangle> {
    if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
        return Vec::new();
    }
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return Vec::new();
    };

    let world_pts: Vec<Vec3> = positions
        .iter()
        .map(|p| transform.transform_point(Vec3::from_array(*p)))
        .collect();

    let mut tris = Vec::new();
    match mesh.indices() {
        Some(Indices::U16(indices)) => {
            for chunk in indices.chunks_exact(3) {
                let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
                if i0 < world_pts.len() && i1 < world_pts.len() && i2 < world_pts.len() {
                    tris.push(StlTriangle::from_vertices(
                        world_pts[i0],
                        world_pts[i1],
                        world_pts[i2],
                    ));
                }
            }
        }
        Some(Indices::U32(indices)) => {
            for chunk in indices.chunks_exact(3) {
                let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
                if i0 < world_pts.len() && i1 < world_pts.len() && i2 < world_pts.len() {
                    tris.push(StlTriangle::from_vertices(
                        world_pts[i0],
                        world_pts[i1],
                        world_pts[i2],
                    ));
                }
            }
        }
        None => {
            for chunk in world_pts.chunks_exact(3) {
                tris.push(StlTriangle::from_vertices(chunk[0], chunk[1], chunk[2]));
            }
        }
    }
    tris
}

#[allow(clippy::type_complexity)]
fn handle_stl_export_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    meshes: Res<Assets<Mesh>>,
    mut status: ResMut<StlExportStatus>,
    mut feed: ResMut<ActivityFeed>,
    shell_query: Query<
        (&Mesh3d, &GlobalTransform),
        (
            Or<(With<ExteriorShell>, With<TerrainPlinth>)>,
            Without<TemporaryProp>,
        ),
    >,
) {
    if !keys.just_pressed(KeyCode::F12) && !status.trigger_requested {
        return;
    }
    status.trigger_requested = false;

    let mut all_triangles = Vec::new();
    for (mesh_h, global_tf) in &shell_query {
        if let Some(mesh) = meshes.get(&mesh_h.0) {
            let mut mesh_tris = extract_world_triangles_from_mesh(mesh, global_tf);
            all_triangles.append(&mut mesh_tris);
        }
    }

    let stl_bytes = serialize_binary_stl(&all_triangles);
    let tri_count = all_triangles.len();
    let formatted_count = format_with_commas(tri_count);

    match File::create(STL_OUTPUT_PATH).and_then(|f| {
        let mut writer = BufWriter::new(f);
        writer.write_all(&stl_bytes)?;
        writer.flush()
    }) {
        Ok(()) => {
            let msg = format!(
                "Royal Charter & 3D Printable Diorama saved to {STL_OUTPUT_PATH}! ({formatted_count} solid triangles)"
            );
            status.last_banner_message = msg.clone();
            status.banner_timer = 9.0;
            feed.push(msg);
        }
        Err(err) => {
            let msg = format!("Failed to write {STL_OUTPUT_PATH}: {err}");
            status.last_banner_message = msg.clone();
            status.banner_timer = 6.0;
            feed.push(msg);
        }
    }
}

fn tick_stl_banner_timer(time: Res<Time>, mut status: ResMut<StlExportStatus>) {
    if status.banner_timer > 0.0 {
        status.banner_timer = (status.banner_timer - time.delta_secs()).max(0.0);
    }
}

fn format_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (count, ch) in s.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_stl_export_format() {
        let t1 = StlTriangle::from_vertices(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let t2 = StlTriangle::from_vertices(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(2.0, 0.0, 0.0),
        );

        // Verify right-hand rule outward normals
        assert!((t1.normal - Vec3::Z).length() < 1e-5);
        assert!((t2.normal - Vec3::Y).length() < 1e-5);

        let triangles = vec![t1, t2];
        let bytes = serialize_binary_stl(&triangles);

        // Standard binary STL size: 80-byte header + 4-byte count + (2 * 50 bytes) = 184 bytes
        assert_eq!(bytes.len(), 80 + 4 + 2 * 50);

        // Verify little-endian triangle count at bytes 80..84
        let count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
        assert_eq!(count, 2);

        // Verify first triangle normal Z == 1.0f32 at bytes 92..96
        let nz = f32::from_le_bytes([bytes[92], bytes[93], bytes[94], bytes[95]]);
        assert!((nz - 1.0).abs() < 1e-5);
    }
}

