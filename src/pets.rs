use crate::blueprint::TemporaryProp;
use crate::camera::CameraMode;
use crate::economy::CozyNeeds;
use crate::npc::{ActivityFeed, LordArchetype, LordMind, Peasant};
use bevy::prelude::*;
use std::f32::consts::TAU;

pub struct PetsPlugin;

impl Plugin for PetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hound_and_grazing_sheep)
            .add_systems(
                Update,
                (
                    update_castle_hound_ai,
                    handle_pet_hound_hotkey,
                    update_grazing_sheep_ai,
                ),
            );
    }
}

#[derive(Component)]
pub struct CastleHound {
    pub bark_timer: f32,
    pub jump_timer: f32,
    pub wag_phase: f32,
    pub tail_entity: Entity,
}

#[derive(Component)]
pub struct GrazingSheep {
    pub target_pos: Vec3,
    pub munch_timer: f32,
    pub bob_phase: f32,
    pub seed: f32,
}

fn setup_hound_and_grazing_sheep(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hound_fur_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.66, 0.44, 0.24),
        perceptual_roughness: 0.85,
        ..default()
    });
    let hound_ear_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.22, 0.12),
        perceptual_roughness: 0.88,
        ..default()
    });
    let collar_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.78, 0.18),
        metallic: 0.7,
        ..default()
    });

    // 1. Spawn "Barnaby's Bane" (The Loyal Castle Hound)
    let tail_ent = commands
        .spawn((
            TemporaryProp,
            Mesh3d(meshes.add(Cylinder::new(0.035, 0.28))),
            MeshMaterial3d(hound_fur_mat.clone()),
            Transform::from_xyz(0.0, 0.36, -0.28).with_rotation(Quat::from_rotation_x(-0.75)),
        ))
        .id();

    let hound_id = commands
        .spawn((
            TemporaryProp,
            CastleHound {
                bark_timer: 6.5,
                jump_timer: 0.0,
                wag_phase: 0.0,
                tail_entity: tail_ent,
            },
            Transform::from_xyz(-2.2, 0.0, 1.2),
            Visibility::default(),
        ))
        .with_children(|dog| {
            // Body
            dog.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cuboid::new(0.28, 0.26, 0.54))),
                MeshMaterial3d(hound_fur_mat.clone()),
                Transform::from_xyz(0.0, 0.26, 0.0),
            ));
            // Head & Snout
            dog.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Sphere::new(0.16))),
                MeshMaterial3d(hound_fur_mat.clone()),
                Transform::from_xyz(0.0, 0.44, 0.26),
            ));
            dog.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cuboid::new(0.13, 0.11, 0.18))),
                MeshMaterial3d(hound_ear_mat.clone()),
                Transform::from_xyz(0.0, 0.41, 0.38),
            ));
            // Royal Gold Collar
            dog.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cylinder::new(0.14, 0.06))),
                MeshMaterial3d(collar_mat),
                Transform::from_xyz(0.0, 0.36, 0.19),
            ));
            // Floppy Ears
            for sx in [-0.13_f32, 0.13_f32] {
                dog.spawn((
                    TemporaryProp,
                    Mesh3d(meshes.add(Cuboid::new(0.06, 0.16, 0.10))),
                    MeshMaterial3d(hound_ear_mat.clone()),
                    Transform::from_xyz(sx, 0.42, 0.24),
                ));
            }
            // 4 Paws
            let leg_mesh = meshes.add(Cylinder::new(0.045, 0.20));
            for (lx, lz) in [(-0.10, -0.18), (0.10, -0.18), (-0.10, 0.18), (0.10, 0.18)] {
                dog.spawn((
                    TemporaryProp,
                    Mesh3d(leg_mesh.clone()),
                    MeshMaterial3d(hound_fur_mat.clone()),
                    Transform::from_xyz(lx, 0.10, lz),
                ));
            }
        })
        .id();
    commands.entity(hound_id).add_child(tail_ent);

    // 2. Spawn 4 Fluffy Grazing Sheep wandering the sunlit meadow
    let wool_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.94, 0.89),
        perceptual_roughness: 0.95,
        ..default()
    });
    let sheep_face_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.22, 0.20),
        perceptual_roughness: 0.9,
        ..default()
    });
    let wool_mesh = meshes.add(Sphere::new(0.36));
    let head_mesh = meshes.add(Sphere::new(0.18));
    let leg_mesh = meshes.add(Cylinder::new(0.045, 0.22));

    let sheep_starts = [
        Vec3::new(-9.5, 0.0, 3.5),
        Vec3::new(-8.0, 0.0, 5.2),
        Vec3::new(9.2, 0.0, -3.5),
        Vec3::new(10.8, 0.0, -1.8),
    ];

    for (i, start) in sheep_starts.into_iter().enumerate() {
        commands
            .spawn((
                TemporaryProp,
                GrazingSheep {
                    target_pos: start + Vec3::new(1.8, 0.0, -1.4),
                    munch_timer: 1.5 + i as f32,
                    bob_phase: i as f32 * 1.3,
                    seed: (i as f32 + 1.0) * 2.7,
                },
                Transform::from_translation(start),
                Visibility::default(),
            ))
            .with_children(|sheep| {
                sheep.spawn((
                    TemporaryProp,
                    Mesh3d(wool_mesh.clone()),
                    MeshMaterial3d(wool_mat.clone()),
                    Transform::from_xyz(0.0, 0.36, 0.0).with_scale(Vec3::new(1.0, 0.88, 1.32)),
                ));
                sheep.spawn((
                    TemporaryProp,
                    Mesh3d(head_mesh.clone()),
                    MeshMaterial3d(sheep_face_mat.clone()),
                    Transform::from_xyz(0.0, 0.42, 0.38),
                ));
                for (lx, lz) in [(-0.14, -0.22), (0.14, -0.22), (-0.14, 0.22), (0.14, 0.22)] {
                    sheep.spawn((
                        TemporaryProp,
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(sheep_face_mat.clone()),
                        Transform::from_xyz(lx, 0.11, lz),
                    ));
                }
            });
    }
}

fn update_castle_hound_ai(
    time: Res<Time>,
    mut feed: ResMut<ActivityFeed>,
    lords_q: Query<(&LordMind, &Transform), Without<CastleHound>>,
    mut peasants_q: Query<(&mut Peasant, &mut CozyNeeds, &Transform), Without<CastleHound>>,
    mut hound_q: Query<(&mut CastleHound, &mut Transform)>,
    mut tail_tf_q: Query<&mut Transform, (Without<CastleHound>, Without<LordMind>, Without<Peasant>)>,
) {
    let dt = time.delta_secs();
    let Ok((mut hound, mut h_tf)) = hound_q.get_single_mut() else { return };

    // Joyfully wag tail continuously
    hound.wag_phase += dt * 16.0;
    if let Ok(mut tail_tf) = tail_tf_q.get_mut(hound.tail_entity) {
        tail_tf.rotation = Quat::from_rotation_x(-0.75) * Quat::from_rotation_z(hound.wag_phase.sin() * 0.55);
    }

    // Follow High Lord Aldric
    if let Some((_, aldric_tf)) = lords_q
        .iter()
        .find(|(l, _)| l.archetype == LordArchetype::PracticalFarmerLord)
    {
        let follow_spot = Vec3::new(aldric_tf.translation.x - 0.85, 0.0, aldric_tf.translation.z - 0.85);
        let cur = Vec3::new(h_tf.translation.x, 0.0, h_tf.translation.z);
        let delta = follow_spot - cur;
        if delta.length() > 0.55 {
            let dir = delta.normalize();
            let next = cur + dir * (4.2 * dt).min(delta.length());
            h_tf.translation.x = next.x;
            h_tf.translation.z = next.z;
            h_tf.look_to(dir, Vec3::Y);
        }
    }

    // Happy jump physics (when petted or barking)
    if hound.jump_timer > 0.0 {
        hound.jump_timer = (hound.jump_timer - dt).max(0.0);
        let jump_arc = ((hound.jump_timer / 0.65) * std::f32::consts::PI).sin();
        h_tf.translation.y = jump_arc * 0.85;
    } else {
        h_tf.translation.y = (hound.wag_phase * 0.5).sin().abs() * 0.08;
    }

    // Periodically bark happily at nearby peasants to boost their morale!
    hound.bark_timer -= dt;
    if hound.bark_timer <= 0.0 {
        hound.bark_timer = 11.0;
        let hound_pos = h_tf.translation;
        let mut cheered_any = false;
        for (mut p, mut cozy, p_tf) in &mut peasants_q {
            if p_tf.translation.distance(hound_pos) <= 9.5 {
                cozy.hound_morale_timer = 14.0;
                cozy.energy = (cozy.energy + 12.0).min(100.0);
                p.hop_timer = 0.40;
                cheered_any = true;
            }
        }
        if cheered_any {
            hound.jump_timer = 0.65;
            feed.push("Barnaby's Bane (Castle Hound) gave a joyful 'WOOF!', cheering nearby peasants (+18% Morale Speed)!".to_string());
        }
    }
}

fn handle_pet_hound_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    cam_mode: Res<CameraMode>,
    mut feed: ResMut<ActivityFeed>,
    mut lords_q: Query<&mut LordMind>,
    mut hound_q: Query<&mut CastleHound>,
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    if matches!(*cam_mode, CameraMode::FirstPersonLord(_)) {
        if let Ok(mut hound) = hound_q.get_single_mut() {
            hound.jump_timer = 0.65;
        }
        for mut lord in &mut lords_q {
            lord.ego = 100.0;
            lord.whisper_influence = 100.0;
            if lord.archetype == LordArchetype::PracticalFarmerLord {
                lord.current_thought = "\"Who's a loyal royal hound?! Barnaby's Bane always restores my spirits!\"".to_string();
            }
        }
        feed.push("PET THE CASTLE HOUND [F]: Barnaby's Bane did a happy spin! Lord Ego & Whisper Influence restored to 100%!".to_string());
    }
}

fn update_grazing_sheep_ai(time: Res<Time>, mut sheep_q: Query<(&mut GrazingSheep, &mut Transform)>) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    for (mut sheep, mut tf) in &mut sheep_q {
        sheep.bob_phase += dt * 4.5;
        let cur = Vec3::new(tf.translation.x, 0.0, tf.translation.z);
        let delta = sheep.target_pos - cur;
        if delta.length() > 0.35 {
            let dir = delta.normalize();
            let next = cur + dir * (0.95 * dt).min(delta.length());
            tf.translation = Vec3::new(next.x, sheep.bob_phase.sin().abs() * 0.06, next.z);
            tf.look_to(dir, Vec3::Y);
        } else {
            // Gently bob head while munching meadow grass
            tf.translation.y = (sheep.bob_phase * 1.4).sin().abs() * 0.04;
            sheep.munch_timer -= dt;
            if sheep.munch_timer <= 0.0 {
                sheep.munch_timer = 4.5;
                let angle = (elapsed * 0.35 + sheep.seed) % TAU;
                let radius = 8.5 + (sheep.seed.sin().abs() * 4.0);
                sheep.target_pos = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
            }
        }
    }
}

