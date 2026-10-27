use crate::blueprint::{BlueprintStrokeState, BuiltStructure, ConstructionSite, DirectorMode, TowerFoundation};
use crate::npc::{ActivityFeed, LordMind, Peasant};
use crate::world::FARMHOUSE_POS;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::f32::consts::FRAC_PI_2;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraMode>()
            .insert_resource(AmbientLight {
                color: Color::srgb(0.88, 0.93, 1.0),
                brightness: 420.0,
            })
            .add_systems(Startup, (setup_camera, setup_meadow_lighting_and_ground))
            .add_systems(Update, update_director_or_fp_camera);
    }
}

#[derive(Component)]
pub struct TerrainPlinth;

#[derive(Component)]
pub struct ExteriorShell;

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum CameraMode {
    #[default]
    DirectorRTS,
    FirstPersonLord(Entity),
}

#[derive(Component)]
pub struct LordLookDirection {
    pub forward: Vec3,
}

impl Default for LordLookDirection {
    fn default() -> Self {
        Self { forward: Vec3::Z }
    }
}

#[derive(Component)]
pub struct DirectorCamera {
    pub focus: Vec3,
    pub target_focus: Vec3,
    pub radius: f32,
    pub target_radius: f32,
    pub yaw: f32,
    pub target_yaw: f32,
    pub pitch: f32,
    pub target_pitch: f32,
    pub pan_speed: f32,
    pub orbit_speed: f32,
    pub zoom_speed: f32,
    pub fp_yaw_offset: f32,
    pub fp_pitch_offset: f32,
    pub grab_anchor_world: Option<Vec3>,
    pub last_click_time: f32,
}

impl Default for DirectorCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::new(0.0, 0.0, 2.0),
            target_focus: Vec3::new(0.0, 0.0, 2.0),
            radius: 34.0,
            target_radius: 34.0,
            yaw: 0.45,
            target_yaw: 0.45,
            pitch: 0.68,
            target_pitch: 0.68,
            pan_speed: 24.0,
            orbit_speed: 2.0,
            zoom_speed: 4.2,
            fp_yaw_offset: 0.0,
            fp_pitch_offset: -0.08,
            grab_anchor_world: None,
            last_click_time: -10.0,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    let cam = DirectorCamera::default();
    let offset = compute_camera_offset(cam.radius, cam.yaw, cam.pitch);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(cam.focus + offset).looking_at(cam.focus, Vec3::Y),
        cam,
    ));
}

fn setup_meadow_lighting_and_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 13_500.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.96, 0.88),
            ..default()
        },
        Transform::from_xyz(38.0, 56.0, 32.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let meadow_mesh = meshes.add(Cuboid::new(180.0, 2.0, 180.0));
    let meadow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.56, 0.25),
        perceptual_roughness: 0.94,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((TerrainPlinth, Mesh3d(meadow_mesh), MeshMaterial3d(meadow_mat), Transform::from_xyz(0.0, -1.0, 0.0)));

    let clearing_mesh = meshes.add(Cylinder::new(15.5, 0.06));
    let clearing_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.61, 0.28),
        perceptual_roughness: 0.95,
        ..default()
    });
    commands.spawn((TerrainPlinth, Mesh3d(clearing_mesh), MeshMaterial3d(clearing_mat), Transform::from_xyz(0.0, 0.005, 0.0)));

    let path_mesh = meshes.add(Cuboid::new(10.5, 0.08, 2.6));
    let path_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.42, 0.29),
        perceptual_roughness: 0.98,
        ..default()
    });
    commands.spawn((TerrainPlinth, Mesh3d(path_mesh), MeshMaterial3d(path_mat), Transform::from_xyz(0.0, 0.01, -1.8)));
}

fn compute_camera_offset(radius: f32, yaw: f32, pitch: f32) -> Vec3 {
    let horiz = radius * pitch.cos();
    let vert = radius * pitch.sin();
    Vec3::new(horiz * yaw.sin(), vert, horiz * yaw.cos())
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_director_or_fp_camera(
    time: Res<Time<Real>>,
    cam_mode: Res<CameraMode>,
    dir_mode: Res<DirectorMode>,
    stroke: Res<BlueprintStrokeState>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut feed: ResMut<ActivityFeed>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    lord_q: Query<(&Transform, &LordLookDirection, &LordMind), Without<DirectorCamera>>,
    peasants_q: Query<(&Transform, &Peasant), Without<DirectorCamera>>,
    sites_q: Query<&ConstructionSite>,
    built_q: Query<&BuiltStructure>,
    towers_q: Query<&TowerFoundation>,
    mut cam_q: Query<(&Camera, &GlobalTransform, &mut DirectorCamera, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.1);
    let now = time.elapsed_secs();
    let Ok((camera, cam_gtf, mut cam, mut transform)) = cam_q.get_single_mut() else { return };

    let mut motion_delta = Vec2::ZERO;
    for ev in mouse_motion.read() {
        motion_delta += ev.delta;
    }

    // --- 1. FIRST-PERSON "IN THEIR EYES" CAMERA MODE ---
    if let CameraMode::FirstPersonLord(lord_entity) = *cam_mode {
        mouse_wheel.clear();
        if let Ok((lord_tf, look_dir, _)) = lord_q.get(lord_entity) {
            cam.fp_yaw_offset = (cam.fp_yaw_offset - motion_delta.x * 0.0035).clamp(-1.15, 1.15);
            cam.fp_pitch_offset = (cam.fp_pitch_offset - motion_delta.y * 0.003).clamp(-0.65, 0.55);
            if keys.pressed(KeyCode::KeyQ) { cam.fp_yaw_offset = (cam.fp_yaw_offset + 1.4 * dt).clamp(-1.15, 1.15); }
            if keys.pressed(KeyCode::KeyE) { cam.fp_yaw_offset = (cam.fp_yaw_offset - 1.4 * dt).clamp(-1.15, 1.15); }
            if motion_delta.length_squared() < 0.01 && !keys.pressed(KeyCode::KeyQ) && !keys.pressed(KeyCode::KeyE) {
                cam.fp_yaw_offset *= (1.0 - 1.5 * dt).max(0.0);
            }
            let eye_pos = lord_tf.translation + Vec3::new(0.0, 1.7, 0.0);
            let base_fwd = Vec3::new(look_dir.forward.x, 0.0, look_dir.forward.z).normalize_or_zero();
            let base_fwd = if base_fwd == Vec3::ZERO { Vec3::Z } else { base_fwd };
            let final_dir = (Quat::from_rotation_y(cam.fp_yaw_offset) * base_fwd + Vec3::Y * cam.fp_pitch_offset).normalize();
            let smooth = (16.0 * dt).min(1.0);
            transform.translation = transform.translation.lerp(eye_pos, smooth);
            let target_rot = Transform::from_translation(eye_pos).looking_to(final_dir, Vec3::Y).rotation;
            transform.rotation = transform.rotation.slerp(target_rot, (12.0 * dt).min(1.0));
            cam.target_focus = Vec3::new(lord_tf.translation.x, 0.0, lord_tf.translation.z);
            cam.focus = cam.target_focus;
            return;
        }
    }

    // Raycast cursor against ground plane
    let cursor_ground_pt = window_q
        .get_single()
        .ok()
        .and_then(|w| w.cursor_position())
        .and_then(|cp| camera.viewport_to_world(cam_gtf, cp).ok())
        .and_then(|ray| ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)).map(|d| ray.get_point(d)));

    // --- 2. DOUBLE-CLICK FOCUS ON LORD, PEASANT, OR STRUCTURE ---
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if now - cam.last_click_time <= 0.32 {
            if let Some(pt) = cursor_ground_pt {
                let mut best_focus: Option<(Vec3, f32, String)> = None;
                for (l_tf, _, l) in &lord_q {
                    if l_tf.translation.distance(pt) <= 2.4 {
                        best_focus = Some((l_tf.translation, 14.5, format!("Camera framed on {}", l.name)));
                    }
                }
                if best_focus.is_none() {
                    for (p_tf, p) in &peasants_q {
                        if p_tf.translation.distance(pt) <= 2.2 {
                            best_focus = Some((p_tf.translation, 14.0, format!("Camera framed on peasant {}", p.name)));
                        }
                    }
                }
                if best_focus.is_none() {
                    for t in &towers_q {
                        if t.center.distance(pt) <= t.radius + 1.5 {
                            best_focus = Some((t.center, 20.0, "Camera framed on Tower Keep Foundation".into()));
                        }
                    }
                }
                if best_focus.is_none() {
                    if let Some(b) = built_q.iter().find(|b| b.position.distance(pt) <= 2.2) {
                        best_focus = Some((b.position, 16.5, "Camera framed on Palisade Wall".into()));
                    } else if let Some(s) = sites_q.iter().find(|s| s.position.distance(pt) <= 2.2) {
                        best_focus = Some((s.position, 16.5, "Camera framed on Blueprint Stake".into()));
                    } else if FARMHOUSE_POS.distance(pt) <= 4.0 {
                        best_focus = Some((FARMHOUSE_POS, 19.0, "Camera framed on Central Farmhouse".into()));
                    }
                }
                if let Some((focus_pt, zoom_r, msg)) = best_focus {
                    cam.target_focus = Vec3::new(focus_pt.x, 0.0, focus_pt.z);
                    cam.target_radius = zoom_r;
                    cam.target_pitch = auto_pitch_for_radius(zoom_r);
                    feed.push(msg);
                }
            }
        }
        cam.last_click_time = now;
    }

    // --- 3. TABLETOP "GRAB & SLIDE" WORLD PAN ---
    let grabbing = mouse_buttons.pressed(MouseButton::Middle)
        || (*dir_mode == DirectorMode::Inspect && mouse_buttons.pressed(MouseButton::Left));
    if grabbing {
        if let Some(pt) = cursor_ground_pt {
            match cam.grab_anchor_world {
                None => cam.grab_anchor_world = Some(pt),
                Some(anchor) => {
                    let slide = Vec3::new(anchor.x - pt.x, 0.0, anchor.z - pt.z);
                    cam.target_focus = (cam.target_focus + slide * 0.65).clamp(Vec3::splat(-65.0), Vec3::splat(65.0));
                    cam.target_focus.y = 0.0;
                }
            }
        }
    } else {
        cam.grab_anchor_world = None;
    }

    // --- 4. ORBIT (Q/E or Right-Drag in Inspect Mode) ---
    if keys.pressed(KeyCode::KeyQ) { cam.target_yaw -= cam.orbit_speed * dt; }
    if keys.pressed(KeyCode::KeyE) { cam.target_yaw += cam.orbit_speed * dt; }
    if *dir_mode == DirectorMode::Inspect && mouse_buttons.pressed(MouseButton::Right) {
        cam.target_yaw -= motion_delta.x * 0.0065;
        cam.target_pitch = (cam.target_pitch + motion_delta.y * 0.0055).clamp(0.22, FRAC_PI_2 - 0.08);
    }

    // --- 5. ZOOM-TO-CURSOR & AUTO-PITCH ---
    if !stroke.hovering_tower_handle {
        for ev in mouse_wheel.read() {
            let scroll = ev.y.clamp(-3.0, 3.0);
            let old_r = cam.target_radius;
            let new_r = (old_r - scroll * cam.zoom_speed).clamp(9.0, 78.0);
            cam.target_radius = new_r;
            // Zoom toward the 3D ground point under the cursor
            if let Some(cursor_pt) = cursor_ground_pt {
                if scroll > 0.0 {
                    let zoom_frac = ((old_r - new_r) / old_r).clamp(0.0, 0.35);
                    cam.target_focus = cam.target_focus.lerp(Vec3::new(cursor_pt.x, 0.0, cursor_pt.z), zoom_frac);
                }
            }
            // Auto-Pitch: horizontal when close to ground, top-down when zoomed out
            cam.target_pitch = auto_pitch_for_radius(new_r);
        }
    } else {
        mouse_wheel.clear();
    }

    // --- 6. WASD / ARROW KEYS PAN ---
    let mut pan_input = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) { pan_input.y -= 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) { pan_input.y += 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) { pan_input.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { pan_input.x += 1.0; }
    if pan_input.length_squared() > 0.0 {
        let dir = pan_input.normalize();
        let forward = Vec3::new(cam.yaw.sin(), 0.0, cam.yaw.cos());
        let right = Vec3::new(cam.yaw.cos(), 0.0, -cam.yaw.sin());
        let zoom_scale = (cam.radius / 30.0).clamp(0.5, 2.2);
        let delta_move = (right * dir.x + forward * dir.y) * cam.pan_speed * zoom_scale * dt;
        cam.target_focus = (cam.target_focus + delta_move).clamp(Vec3::splat(-65.0), Vec3::splat(65.0));
        cam.target_focus.y = 0.0;
    }

    // --- 7. SILKY SMOOTH LERP DAMPING ---
    let smooth = (11.5 * dt).min(1.0);
    cam.focus = cam.focus.lerp(cam.target_focus, smooth);
    cam.radius = cam.radius + (cam.target_radius - cam.radius) * smooth;
    cam.yaw = cam.yaw + (cam.target_yaw - cam.yaw) * smooth;
    cam.pitch = cam.pitch + (cam.target_pitch - cam.pitch) * smooth;

    let offset = compute_camera_offset(cam.radius, cam.yaw, cam.pitch);
    transform.translation = cam.focus + offset;
    transform.look_at(cam.focus, Vec3::Y);
}

fn auto_pitch_for_radius(radius: f32) -> f32 {
    let t = ((radius - 9.0) / (78.0 - 9.0)).clamp(0.0, 1.0);
    0.28 + t * 0.78
}

