use crate::camera::ExteriorShell;
use crate::npc::{LordMind, Peasant};
use crate::state::{HistoricalEra, HistoricalEraState};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct ProceduralConstructionPlugin;
impl Plugin for ProceduralConstructionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueprintStats>()
            .add_systems(Startup, setup_blueprint_assets_and_starter_stakes)
            .add_systems(Update, (animate_staking_pop, animate_rising_palisade_posts, animate_scaffolding_puffs, animate_gatehouse_doors, draw_tower_and_priority_gizmos));
    }
}

#[derive(Resource, Default)]
pub struct BlueprintStats { pub stakes_planned: u32, pub posts_completed: u32 }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SiteKind { PalisadePost, GatehouseArchway, TowerKeepWall { stories: u32 } }
#[derive(Component)]
pub struct TemporaryProp;
#[derive(Component)]
pub struct ScaffoldingPuff { pub elapsed: f32 }
#[derive(Component)]
pub struct GateDoor { pub center_pos: Vec3, pub side_sign: f32, pub current_angle: f32 }

#[derive(Component)]
pub struct ConstructionSite {
    pub position: Vec3, pub prev_stake_pos: Option<Vec3>, pub stroke_id: u32, pub kind: SiteKind,
    pub priority_bell: bool, pub has_log: bool, pub ditch_dug: bool,
    pub reserved_log_delivery: Option<Entity>, pub reserved_builder: Option<Entity>,
    pub build_progress: f32, pub build_required: f32,
    pub delivered_log_vis: Option<Entity>, pub scaffolding_vis: Option<Entity>,
}

#[derive(Component, Clone, Copy)]
pub struct BuiltStructure { pub position: Vec3, pub prev_pos: Option<Vec3>, pub stroke_id: u32, pub kind: SiteKind, pub is_stone: bool }
#[derive(Component)]
pub struct TowerFoundation { pub center: Vec3, pub radius: f32, pub stories: u32, pub stroke_id: u32, pub priority_bell: bool }
#[derive(Component)]
pub struct StakingPopAnimation { pub elapsed: f32 }
#[derive(Component)]
pub struct RisingPalisadePost { pub target_y: f32, pub current_y: f32 }

#[derive(Resource)]
pub struct BlueprintAssets {
    pub stake_mesh: Handle<Mesh>, pub stake_tip_mesh: Handle<Mesh>,
    pub stake_mat: Handle<StandardMaterial>, pub rope_mat: Handle<StandardMaterial>, pub gate_mat: Handle<StandardMaterial>,
    pub post_mesh: Handle<Mesh>, pub post_tip_mesh: Handle<Mesh>, pub post_mat: Handle<StandardMaterial>,
    pub stone_mat: Handle<StandardMaterial>, pub roof_mat: Handle<StandardMaterial>, pub slate_mat: Handle<StandardMaterial>,
    pub ditch_mound_mat: Handle<StandardMaterial>, pub trench_mat: Handle<StandardMaterial>,
    pub site_log_mesh: Handle<Mesh>, pub puff_mesh: Handle<Mesh>,
}

fn setup_blueprint_assets_and_starter_stakes(
    mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>, mut stats: ResMut<BlueprintStats>,
) {
    let assets = BlueprintAssets {
        stake_mesh: meshes.add(Cylinder::new(0.065, 0.75)),
        stake_tip_mesh: meshes.add(Cone { radius: 0.08, height: 0.22 }),
        stake_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.78, 0.63, 0.36), perceptual_roughness: 0.85, ..default() }),
        rope_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.92, 0.84, 0.58), emissive: LinearRgba::new(0.18, 0.15, 0.06, 1.0), ..default() }),
        gate_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.95, 0.72, 0.24), emissive: LinearRgba::new(0.28, 0.18, 0.02, 1.0), ..default() }),
        post_mesh: meshes.add(Cylinder::new(0.24, 2.55)),
        post_tip_mesh: meshes.add(Cone { radius: 0.25, height: 0.55 }),
        post_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.44, 0.30, 0.17), perceptual_roughness: 0.88, ..default() }),
        stone_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.62, 0.60, 0.56), perceptual_roughness: 0.88, ..default() }),
        roof_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.66, 0.28, 0.20), perceptual_roughness: 0.85, ..default() }),
        slate_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.26, 0.32, 0.42), perceptual_roughness: 0.75, ..default() }),
        ditch_mound_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.40, 0.31, 0.20), perceptual_roughness: 0.96, ..default() }),
        trench_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.24, 0.18, 0.11), perceptual_roughness: 0.99, ..default() }),
        site_log_mesh: meshes.add(Cylinder::new(0.15, 1.25)),
        puff_mesh: meshes.add(Sphere::new(0.55)),
    };

    // Starter Inner Ring Southern Arc (Radius 11.0m, with a central Gatehouse Archway)
    let starter = [
        (Vec3::new(-4.2, 0.0, 10.1), SiteKind::PalisadePost),
        (Vec3::new(-2.1, 0.0, 10.8), SiteKind::PalisadePost),
        (Vec3::new(0.0, 0.0, 11.0), SiteKind::GatehouseArchway),
        (Vec3::new(2.1, 0.0, 10.8), SiteKind::PalisadePost),
        (Vec3::new(4.2, 0.0, 10.1), SiteKind::PalisadePost),
    ];
    let mut prev = None;
    for (pt, kind) in starter {
        spawn_stake_site(&mut commands, &mut meshes, &assets, pt, prev, 1, kind);
        stats.stakes_planned += 1;
        prev = Some(pt);
    }
    commands.insert_resource(assets);
}

pub fn spawn_stake_site(
    commands: &mut Commands, meshes: &mut Assets<Mesh>, assets: &BlueprintAssets,
    pos: Vec3, prev_stake_pos: Option<Vec3>, stroke_id: u32, kind: SiteKind,
) -> Entity {
    let is_gate = matches!(kind, SiteKind::GatehouseArchway);
    commands.spawn((
        ConstructionSite {
            position: pos, prev_stake_pos, stroke_id, kind, priority_bell: false,
            has_log: false, ditch_dug: false, reserved_log_delivery: None, reserved_builder: None,
            build_progress: 0.0, build_required: 3.0, delivered_log_vis: None, scaffolding_vis: None,
        },
        StakingPopAnimation { elapsed: 0.0 },
        Transform::from_translation(pos).with_scale(Vec3::splat(0.1)),
        Visibility::default(),
    )).with_children(|parent| {
        if is_gate {
            for side in [-0.65_f32, 0.65_f32] {
                parent.spawn((Mesh3d(assets.stake_mesh.clone()), MeshMaterial3d(assets.gate_mat.clone()), Transform::from_xyz(side, 0.55, 0.0).with_scale(Vec3::new(1.4, 1.45, 1.4))));
            }
            parent.spawn((Mesh3d(meshes.add(Cuboid::new(1.5, 0.08, 0.08))), MeshMaterial3d(assets.gate_mat.clone()), Transform::from_xyz(0.0, 1.15, 0.0)));
        } else {
            parent.spawn((Mesh3d(assets.stake_mesh.clone()), MeshMaterial3d(assets.stake_mat.clone()), Transform::from_xyz(0.0, 0.375, 0.0)));
            parent.spawn((Mesh3d(assets.stake_tip_mesh.clone()), MeshMaterial3d(assets.rope_mat.clone()), Transform::from_xyz(0.0, 0.78, 0.0)));
        }
        if let Some(prev) = prev_stake_pos {
            let delta = prev - pos;
            let dist = delta.length();
            if dist > 0.1 && dist < 4.5 {
                let mid = delta * 0.5 + Vec3::new(0.0, 0.58, 0.0);
                let rot = Quat::from_rotation_arc(Vec3::Y, delta.normalize());
                parent.spawn((Mesh3d(meshes.add(Cylinder::new(0.022, dist))), MeshMaterial3d(assets.rope_mat.clone()), Transform::from_translation(mid).with_rotation(rot)));
            }
        }
    }).id()
}

pub fn spawn_delivered_log_on_site(commands: &mut Commands, assets: &BlueprintAssets, site_pos: Vec3) -> Entity {
    commands.spawn((
        TemporaryProp, Mesh3d(assets.site_log_mesh.clone()), MeshMaterial3d(assets.post_mat.clone()),
        Transform::from_translation(site_pos + Vec3::new(0.0, 0.16, 0.32)).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    )).id()
}

pub fn spawn_ditch_and_scaffolding_stage(
    commands: &mut Commands, meshes: &mut Assets<Mesh>, assets: &BlueprintAssets,
    site: &mut ConstructionSite, era: HistoricalEra,
) {
    site.ditch_dug = true;
    let pos = site.position;
    commands.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(1.55, 0.08, 0.55))), MeshMaterial3d(assets.trench_mat.clone()), Transform::from_translation(pos + Vec3::new(0.0, 0.01, 0.52))));
    commands.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(1.6, 0.36, 0.75))), MeshMaterial3d(assets.ditch_mound_mat.clone()), Transform::from_translation(pos + Vec3::new(0.0, 0.16, -0.35)).with_rotation(Quat::from_rotation_x(-0.22))));

    if (era.uses_stone_masonry() || !matches!(site.kind, SiteKind::PalisadePost)) && site.scaffolding_vis.is_none() {
        let h = match site.kind { SiteKind::TowerKeepWall { stories } => (stories as f32) * 1.9, _ => 2.6 };
        let scaff = commands.spawn((TemporaryProp, Transform::from_translation(pos), Visibility::default())).with_children(|p| {
            let pole_mesh = meshes.add(Cylinder::new(0.045, h));
            let plank_mesh = meshes.add(Cuboid::new(1.35, 0.06, 0.95));
            for (x, z) in [(-0.55, -0.42), (0.55, -0.42), (-0.55, 0.42), (0.55, 0.42)] {
                p.spawn((TemporaryProp, Mesh3d(pole_mesh.clone()), MeshMaterial3d(assets.stake_mat.clone()), Transform::from_xyz(x, h * 0.5, z)));
            }
            p.spawn((TemporaryProp, Mesh3d(plank_mesh.clone()), MeshMaterial3d(assets.stake_mat.clone()), Transform::from_xyz(0.0, h * 0.48, 0.0)));
            p.spawn((TemporaryProp, Mesh3d(plank_mesh), MeshMaterial3d(assets.stake_mat.clone()), Transform::from_xyz(0.0, h * 0.85, 0.0)));
        }).id();
        site.scaffolding_vis = Some(scaff);
    }
}

pub fn complete_palisade_site(
    commands: &mut Commands, meshes: &mut Assets<Mesh>, assets: &BlueprintAssets,
    site_entity: Entity, site: &ConstructionSite, stats: &mut BlueprintStats, era_state: &HistoricalEraState,
) {
    if let Some(log_vis) = site.delivered_log_vis { commands.entity(log_vis).despawn_recursive(); }
    if let Some(scaff) = site.scaffolding_vis {
        commands.entity(scaff).despawn_recursive();
        commands.spawn((
            TemporaryProp, ScaffoldingPuff { elapsed: 0.0 },
            Mesh3d(assets.puff_mesh.clone()), MeshMaterial3d(assets.rope_mat.clone()),
            Transform::from_translation(site.position + Vec3::Y * 1.4),
        ));
    }
    commands.entity(site_entity).despawn_recursive();
    stats.stakes_planned = stats.stakes_planned.saturating_sub(1);
    stats.posts_completed += 1;
    let built = BuiltStructure {
        position: site.position,
        prev_pos: site.prev_stake_pos,
        stroke_id: site.stroke_id,
        kind: site.kind,
        is_stone: era_state.current.uses_stone_masonry(),
    };
    spawn_built_structure_mesh(commands, meshes, assets, built, era_state.current);
}

pub fn retrofit_timber_structure_to_stone(
    commands: &mut Commands, meshes: &mut Assets<Mesh>, assets: &BlueprintAssets,
    ent: Entity, built: &BuiltStructure, era: HistoricalEra,
) {
    commands.entity(ent).despawn_recursive();
    commands.spawn((
        TemporaryProp, ScaffoldingPuff { elapsed: 0.0 },
        Mesh3d(assets.puff_mesh.clone()), MeshMaterial3d(assets.rope_mat.clone()),
        Transform::from_translation(built.position + Vec3::Y * 1.5),
    ));
    let mut upgraded = *built;
    upgraded.is_stone = true;
    let target_era = if era.uses_stone_masonry() { era } else { HistoricalEra::Era3NormanStoneKeep };
    spawn_built_structure_mesh(commands, meshes, assets, upgraded, target_era);
}

pub fn spawn_built_structure_mesh(
    commands: &mut Commands, meshes: &mut Assets<Mesh>, assets: &BlueprintAssets,
    built: BuiltStructure, era: HistoricalEra,
) {
    let (pos, prev, kind, stone_era) = (built.position, built.prev_pos, built.kind, built.is_stone);
    let wall_mat = if stone_era { assets.stone_mat.clone() } else { assets.post_mat.clone() };
    let roof_mat = if era >= HistoricalEra::Era5GrandChateau { assets.slate_mat.clone() } else { assets.roof_mat.clone() };

    commands.spawn((
        built,
        RisingPalisadePost { target_y: 0.0, current_y: -2.8 },
        Transform::from_xyz(pos.x, -2.8, pos.z),
        Visibility::default(),
    )).with_children(|parent| match kind {
        SiteKind::GatehouseArchway => {
            for side in [-0.78_f32, 0.78_f32] {
                parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.68, 3.1, 0.85))), MeshMaterial3d(wall_mat.clone()), Transform::from_xyz(side, 1.55, 0.0)));
            }
            parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(2.35, 0.55, 0.95))), MeshMaterial3d(wall_mat.clone()), Transform::from_xyz(0.0, 2.95, 0.0)));
            parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cone { radius: 1.42, height: 1.05 })), MeshMaterial3d(roof_mat), Transform::from_xyz(0.0, 3.68, 0.0).with_scale(Vec3::new(1.25, 1.0, 0.78))));
            for side_sign in [-1.0_f32, 1.0_f32] {
                parent.spawn((
                    ExteriorShell, GateDoor { center_pos: pos, side_sign, current_angle: 0.0 },
                    Mesh3d(meshes.add(Cuboid::new(0.46, 2.1, 0.12))), MeshMaterial3d(assets.post_mat.clone()),
                    Transform::from_xyz(side_sign * 0.25, 1.05, 0.0),
                ));
            }
        }
        SiteKind::TowerKeepWall { stories } => {
            let h_scale = (stories as f32).clamp(1.0, 4.0) * 0.92;
            parent.spawn((ExteriorShell, Mesh3d(assets.post_mesh.clone()), MeshMaterial3d(wall_mat), Transform::from_xyz(0.0, 1.25 * h_scale, 0.0).with_scale(Vec3::new(1.55, h_scale, 1.55))));
            parent.spawn((ExteriorShell, Mesh3d(assets.post_tip_mesh.clone()), MeshMaterial3d(roof_mat), Transform::from_xyz(0.0, 2.55 * h_scale + 0.28, 0.0).with_scale(Vec3::new(1.65, 1.35, 1.65))));
        }
        SiteKind::PalisadePost => {
            if stone_era {
                parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.68, 2.8, 0.68))), MeshMaterial3d(assets.stone_mat.clone()), Transform::from_xyz(0.0, 1.4, 0.0)));
                parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.42, 0.45, 0.42))), MeshMaterial3d(assets.stone_mat.clone()), Transform::from_xyz(0.0, 3.02, 0.0)));
            } else {
                parent.spawn((ExteriorShell, Mesh3d(assets.post_mesh.clone()), MeshMaterial3d(assets.post_mat.clone()), Transform::from_xyz(0.0, 1.25, 0.0)));
                parent.spawn((ExteriorShell, Mesh3d(assets.post_tip_mesh.clone()), MeshMaterial3d(assets.stake_mat.clone()), Transform::from_xyz(0.0, 2.78, 0.0)));
            }
            if let Some(prev_pos) = prev {
                let delta = prev_pos - pos;
                let dist = delta.length();
                if dist > 0.2 && dist <= 4.6 {
                    let dir = delta.normalize();
                    let yaw_rot = Quat::from_rotation_y(dir.x.atan2(dir.z));
                    if stone_era {
                        parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.58, 2.55, dist))), MeshMaterial3d(assets.stone_mat.clone()), Transform::from_translation(delta * 0.5 + Vec3::Y * 1.275).with_rotation(yaw_rot)));
                        parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.76, 0.45, dist))), MeshMaterial3d(assets.stone_mat.clone()), Transform::from_translation(delta * 0.5 + Vec3::Y * 0.225).with_rotation(yaw_rot)));
                        for frac in [0.33_f32, 0.66_f32] {
                            parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(0.36, 0.42, 0.36))), MeshMaterial3d(assets.stone_mat.clone()), Transform::from_translation(delta * frac + Vec3::Y * 2.76)));
                        }
                    } else {
                        let rot = Quat::from_rotation_arc(Vec3::Y, dir);
                        parent.spawn((ExteriorShell, Mesh3d(meshes.add(Cylinder::new(0.09, dist))), MeshMaterial3d(assets.post_mat.clone()), Transform::from_translation(delta * 0.5 + Vec3::Y * 1.65).with_rotation(rot)));
                        for frac in [0.25_f32, 0.50_f32, 0.75_f32] {
                            let off = delta * frac;
                            parent.spawn((ExteriorShell, Mesh3d(assets.post_mesh.clone()), MeshMaterial3d(assets.post_mat.clone()), Transform::from_xyz(off.x, 1.18, off.z).with_scale(Vec3::new(0.84, 0.94, 0.84))));
                            parent.spawn((ExteriorShell, Mesh3d(assets.post_tip_mesh.clone()), MeshMaterial3d(assets.stake_mat.clone()), Transform::from_xyz(off.x, 2.58, off.z).with_scale(Vec3::splat(0.84))));
                        }
                    }
                }
            }
        }
    });
}

fn animate_staking_pop(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut StakingPopAnimation, &mut Transform)>) {
    let dt = time.delta_secs();
    for (ent, mut pop, mut tf) in &mut query {
        pop.elapsed += dt;
        let t = (pop.elapsed / 0.28).min(1.0);
        tf.scale = Vec3::splat(if t < 0.65 { 0.1 + (t / 0.65) * 1.18 } else { 1.28 - ((t - 0.65) / 0.35) * 0.28 });
        if t >= 1.0 { tf.scale = Vec3::ONE; commands.entity(ent).remove::<StakingPopAnimation>(); }
    }
}

fn animate_rising_palisade_posts(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut RisingPalisadePost, &mut Transform)>) {
    let dt = time.delta_secs();
    for (entity, mut rising, mut tf) in &mut query {
        rising.current_y = (rising.current_y + dt * 5.5).min(rising.target_y);
        tf.translation.y = rising.current_y;
        if (rising.current_y - rising.target_y).abs() < 0.01 { tf.translation.y = rising.target_y; commands.entity(entity).remove::<RisingPalisadePost>(); }
    }
}

fn animate_scaffolding_puffs(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut ScaffoldingPuff, &mut Transform)>) {
    let dt = time.delta_secs();
    for (ent, mut puff, mut tf) in &mut query {
        puff.elapsed += dt;
        let t = (puff.elapsed / 0.42).min(1.0);
        tf.translation.y += dt * 2.2;
        tf.scale = Vec3::splat((1.0 - t) * 1.4);
        if t >= 1.0 { commands.entity(ent).despawn_recursive(); }
    }
}

fn animate_gatehouse_doors(
    time: Res<Time>,
    peasants_q: Query<&Transform, (With<Peasant>, Without<GateDoor>)>,
    lords_q: Query<&Transform, (With<LordMind>, Without<GateDoor>)>,
    mut doors_q: Query<(&mut GateDoor, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut door, mut tf) in &mut doors_q {
        let npc_near = peasants_q.iter().any(|t| t.translation.distance(door.center_pos) < 3.0)
            || lords_q.iter().any(|t| t.translation.distance(door.center_pos) < 3.0);
        let target_angle = if npc_near { door.side_sign * 1.35 } else { 0.0 };
        door.current_angle += (target_angle - door.current_angle) * (6.0 * dt).min(1.0);
        tf.rotation = Quat::from_rotation_y(door.current_angle);
    }
}

fn draw_tower_and_priority_gizmos(time: Res<Time>, mut gizmos: Gizmos, sites: Query<&ConstructionSite>, towers: Query<&TowerFoundation>) {
    let pulse = (time.elapsed_secs() * 6.0).sin() * 0.5 + 0.5;
    let gold = Color::srgb(1.0, 0.86, 0.18);
    for site in &sites {
        if site.priority_bell {
            gizmos.circle(Isometry3d::new(site.position + Vec3::Y * 0.12, Quat::from_rotation_x(FRAC_PI_2)), 0.55 + pulse * 0.35, gold);
            gizmos.circle(Isometry3d::new(site.position + Vec3::Y * (1.2 + pulse * 0.25), Quat::from_rotation_x(FRAC_PI_2)), 0.28, gold);
        }
        if matches!(site.kind, SiteKind::GatehouseArchway) {
            gizmos.circle(Isometry3d::new(site.position + Vec3::Y * 1.15, Quat::from_rotation_x(FRAC_PI_2)), 0.75, Color::srgb(0.98, 0.72, 0.22));
        }
    }
    for tower in &towers {
        let top_y = (tower.stories as f32) * 2.2;
        let handle_pos = tower.center + Vec3::Y * top_y;
        let color = if tower.priority_bell { gold } else { Color::srgb(0.45, 0.88, 1.0) };
        gizmos.circle(Isometry3d::new(tower.center + Vec3::Y * 0.08, Quat::from_rotation_x(FRAC_PI_2)), tower.radius, color);
        gizmos.line(tower.center, handle_pos, color);
        for s in 1..=tower.stories {
            gizmos.circle(Isometry3d::new(tower.center + Vec3::Y * (s as f32 * 2.2), Quat::from_rotation_x(FRAC_PI_2)), 0.55, gold);
        }
        gizmos.sphere(Isometry3d::from_translation(handle_pos), 0.32 + pulse * 0.08, gold);
    }
}
