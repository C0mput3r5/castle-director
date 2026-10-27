use crate::camera::ExteriorShell;
use bevy::prelude::*;
use rand::prelude::*;
use std::f32::consts::{FRAC_PI_2, TAU};

pub const FARMHOUSE_POS: Vec3 = Vec3::new(-4.8, 0.0, -1.8);
pub const STOCKPILE_POS: Vec3 = Vec3::new(4.8, 0.0, -1.8);

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stockpile>()
            .add_systems(Startup, (setup_farmhouse_and_stockpile, scatter_perimeter_trees).chain())
            .add_systems(Update, (sync_stockpile_visual_logs, tick_tree_saplings_regrowth));
    }
}

#[derive(Resource)]
pub struct Stockpile {
    pub timber_logs: u32,
}

impl Default for Stockpile {
    fn default() -> Self {
        Self { timber_logs: 3 }
    }
}

#[derive(Component)]
pub struct StockpileVisualLog {
    pub index: u32,
}

#[derive(Component)]
pub struct Tree {
    pub health: f32,
    pub reserved_by: Option<Entity>,
}

#[derive(Component)]
pub struct TreeSapling {
    pub regrow_timer: f32,
    pub target_scale: f32,
}

#[derive(Component)]
pub struct GroundLog {
    pub reserved_by: Option<Entity>,
}

#[derive(Resource)]
pub struct LogAssets {
    pub bark_mesh: Handle<Mesh>,
    pub end_mesh: Handle<Mesh>,
    pub bark_mat: Handle<StandardMaterial>,
    pub cut_wood_mat: Handle<StandardMaterial>,
    pub tree_trunk_mesh: Handle<Mesh>,
    pub tree_lower_canopy: Handle<Mesh>,
    pub tree_upper_canopy: Handle<Mesh>,
    pub tree_trunk_mat: Handle<StandardMaterial>,
    pub tree_foliage_mat: Handle<StandardMaterial>,
}

fn setup_farmhouse_and_stockpile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stone_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.46, 0.45, 0.43), perceptual_roughness: 0.92, ..default() });
    let daub_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.84, 0.78, 0.66), perceptual_roughness: 0.9, ..default() });
    let timber_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.36, 0.24, 0.14), perceptual_roughness: 0.88, ..default() });
    let thatch_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.68, 0.55, 0.28), perceptual_roughness: 0.95, ..default() });
    let cut_wood_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.76, 0.60, 0.38), perceptual_roughness: 0.85, ..default() });
    let foliage_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.22, 0.47, 0.20), perceptual_roughness: 0.88, ..default() });

    commands.insert_resource(LogAssets {
        bark_mesh: meshes.add(Cylinder::new(0.18, 1.45)),
        end_mesh: meshes.add(Cylinder::new(0.16, 1.47)),
        bark_mat: timber_mat.clone(),
        cut_wood_mat: cut_wood_mat.clone(),
        tree_trunk_mesh: meshes.add(Cylinder::new(0.24, 2.2)),
        tree_lower_canopy: meshes.add(Cone { radius: 1.45, height: 2.6 }),
        tree_upper_canopy: meshes.add(Cone { radius: 1.05, height: 2.1 }),
        tree_trunk_mat: timber_mat.clone(),
        tree_foliage_mat: foliage_mat,
    });

    // --- 1. Central Rudimentary Farmhouse (Watertight ExteriorShell) ---
    commands.spawn((Transform::from_translation(FARMHOUSE_POS), Visibility::default())).with_children(|parent| {
        parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(5.4, 0.45, 3.8))), MeshMaterial3d(stone_mat.clone()), Transform::from_xyz(0.0, 0.225, 0.0)));
        parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(5.0, 2.2, 3.4))), MeshMaterial3d(daub_mat.clone()), Transform::from_xyz(0.0, 1.5, 0.0)));
        for x in [-2.48, 0.0, 2.48] {
            for z in [-1.68, 1.68] {
                parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.18, 2.3, 0.18))), MeshMaterial3d(timber_mat.clone()), Transform::from_xyz(x, 1.5, z)));
            }
        }
        let roof_slope = meshes.add(Cuboid::new(5.8, 0.28, 2.65));
        parent.spawn((ExteriorShell, Mesh3d(roof_slope.clone()), MeshMaterial3d(thatch_mat.clone()), Transform::from_xyz(0.0, 3.25, 0.95).with_rotation(Quat::from_rotation_x(0.58))));
        parent.spawn((ExteriorShell, Mesh3d(roof_slope), MeshMaterial3d(thatch_mat), Transform::from_xyz(0.0, 3.25, -0.95).with_rotation(Quat::from_rotation_x(-0.58))));
        parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cylinder::new(0.14, 6.0))), MeshMaterial3d(timber_mat.clone()), Transform::from_xyz(0.0, 3.92, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2))));
        parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.95, 1.65, 0.12))), MeshMaterial3d(timber_mat.clone()), Transform::from_xyz(0.6, 1.25, 1.72)));
        parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.75, 4.3, 0.75))), MeshMaterial3d(stone_mat), Transform::from_xyz(-2.1, 2.15, -0.9)));
    });

    // --- 2. Resource Stockpile Zone (next to Farmhouse) ---
    let yard_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.45, 0.36, 0.24), perceptual_roughness: 0.96, ..default() });
    commands.spawn((Transform::from_translation(STOCKPILE_POS), Visibility::default())).with_children(|parent| {
        parent.spawn((Mesh3d(meshes.add(Cuboid::new(4.4, 0.12, 3.4))), MeshMaterial3d(yard_mat), Transform::from_xyz(0.0, 0.06, 0.0)));
        for z in [-0.55, 0.55] {
            parent.spawn((Mesh3d(meshes.add(Cuboid::new(3.6, 0.16, 0.2))), MeshMaterial3d(timber_mat.clone()), Transform::from_xyz(0.0, 0.16, z)));
        }
        for (x, z) in [(-1.7, -1.4), (1.7, -1.4), (-1.7, 1.4), (1.7, 1.4)] {
            parent.spawn((Mesh3d(meshes.add(Cylinder::new(0.09, 1.1))), MeshMaterial3d(timber_mat.clone()), Transform::from_xyz(x, 0.55, z)));
        }
    });
}

fn scatter_perimeter_trees(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    log_assets: Res<LogAssets>,
) {
    let mut rng = StdRng::seed_from_u64(1066);
    let foliage_mats = [
        log_assets.tree_foliage_mat.clone(),
        materials.add(StandardMaterial { base_color: Color::srgb(0.25, 0.50, 0.21), perceptual_roughness: 0.88, ..default() }),
        materials.add(StandardMaterial { base_color: Color::srgb(0.17, 0.39, 0.16), perceptual_roughness: 0.88, ..default() }),
    ];

    let tree_count = 28;
    for i in 0..tree_count {
        let base_angle = (i as f32 / tree_count as f32) * TAU;
        let angle = base_angle + rng.gen_range(-0.08..0.08);
        let dist = rng.gen_range(15.5..34.5);
        let scale = rng.gen_range(0.85..1.28);
        let pos = Vec3::new(angle.cos() * dist, 0.0, angle.sin() * dist);
        let mat = foliage_mats[i % foliage_mats.len()].clone();
        spawn_harvestable_tree(&mut commands, &log_assets, pos, scale, mat);
    }
}

pub fn spawn_harvestable_tree(
    commands: &mut Commands,
    log_assets: &LogAssets,
    pos: Vec3,
    scale: f32,
    foliage_mat: Handle<StandardMaterial>,
) {
    commands
        .spawn((
            Tree { health: 4.0, reserved_by: None },
            Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((Mesh3d(log_assets.tree_trunk_mesh.clone()), MeshMaterial3d(log_assets.tree_trunk_mat.clone()), Transform::from_xyz(0.0, 1.1, 0.0)));
            parent.spawn((Mesh3d(log_assets.tree_lower_canopy.clone()), MeshMaterial3d(foliage_mat.clone()), Transform::from_xyz(0.0, 2.7, 0.0)));
            parent.spawn((Mesh3d(log_assets.tree_upper_canopy.clone()), MeshMaterial3d(foliage_mat), Transform::from_xyz(0.0, 3.85, 0.0)));
        });
}

pub fn spawn_tree_sapling(commands: &mut Commands, log_assets: &LogAssets, pos: Vec3) {
    commands
        .spawn((
            TreeSapling { regrow_timer: 75.0, target_scale: 1.05 },
            Transform::from_translation(Vec3::new(pos.x, 0.0, pos.z)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Stump base
            parent.spawn((
                Mesh3d(log_assets.tree_trunk_mesh.clone()),
                MeshMaterial3d(log_assets.tree_trunk_mat.clone()),
                Transform::from_xyz(0.0, 0.16, 0.0).with_scale(Vec3::new(0.95, 0.15, 0.95)),
            ));
            // Tiny green sapling shoot
            parent.spawn((
                Mesh3d(log_assets.tree_upper_canopy.clone()),
                MeshMaterial3d(log_assets.tree_foliage_mat.clone()),
                Transform::from_xyz(0.0, 0.48, 0.0).with_scale(Vec3::splat(0.28)),
            ));
        });
}

fn tick_tree_saplings_regrowth(
    mut commands: Commands,
    time: Res<Time>,
    log_assets: Option<Res<LogAssets>>,
    mut saplings_q: Query<(Entity, &mut TreeSapling, &mut Transform)>,
) {
    let Some(assets) = log_assets else { return };
    let dt = time.delta_secs();
    for (ent, mut sapling, mut tf) in &mut saplings_q {
        sapling.regrow_timer -= dt;
        let progress = (1.0 - (sapling.regrow_timer / 75.0)).clamp(0.0, 1.0);
        tf.scale = Vec3::splat(0.65 + progress * 0.45);
        if sapling.regrow_timer <= 0.0 {
            let pos = Vec3::new(tf.translation.x, 0.0, tf.translation.z);
            let scale = sapling.target_scale;
            commands.entity(ent).despawn_recursive();
            spawn_harvestable_tree(&mut commands, &assets, pos, scale, assets.tree_foliage_mat.clone());
        }
    }
}

pub fn spawn_ground_log(commands: &mut Commands, log_assets: &LogAssets, pos: Vec3, angle: f32) {
    commands
        .spawn((
            GroundLog { reserved_by: None },
            Transform::from_translation(Vec3::new(pos.x, 0.20, pos.z)).with_rotation(Quat::from_rotation_y(angle)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((Mesh3d(log_assets.bark_mesh.clone()), MeshMaterial3d(log_assets.bark_mat.clone()), Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_2))));
            parent.spawn((Mesh3d(log_assets.end_mesh.clone()), MeshMaterial3d(log_assets.cut_wood_mat.clone()), Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_2))));
        });
}

fn sync_stockpile_visual_logs(
    mut commands: Commands,
    stockpile: Res<Stockpile>,
    log_assets: Option<Res<LogAssets>>,
    existing_query: Query<(Entity, &StockpileVisualLog)>,
) {
    if !stockpile.is_changed() {
        return;
    }
    let Some(assets) = log_assets else { return };
    let desired_vis = stockpile.timber_logs.min(24);
    let mut current_count = 0u32;

    for (entity, vis_log) in &existing_query {
        if vis_log.index >= desired_vis {
            commands.entity(entity).despawn_recursive();
        } else {
            current_count += 1;
        }
    }

    if current_count < desired_vis {
        for idx in 0..desired_vis {
            if !existing_query.iter().any(|(_, v)| v.index == idx) {
                let col = (idx % 6) as f32;
                let row = (idx / 6) as f32;
                let local_x = -1.15 + col * 0.44 + (row % 2.0) * 0.12;
                let local_y = 0.36 + row * 0.31;
                let world_pos = STOCKPILE_POS + Vec3::new(local_x, local_y, 0.0);
                commands
                    .spawn((
                        StockpileVisualLog { index: idx },
                        Transform::from_translation(world_pos),
                        Visibility::default(),
                    ))
                    .with_children(|parent| {
                        parent.spawn((Mesh3d(assets.bark_mesh.clone()), MeshMaterial3d(assets.bark_mat.clone()), Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2))));
                    });
            }
        }
    }
}
