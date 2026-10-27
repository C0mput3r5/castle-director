use crate::blueprint::TemporaryProp;
use crate::camera::ExteriorShell;
use crate::economy::{CozyNeeds, HomesteadAutomatedGuilds};
use crate::npc::{ActivityFeed, LordMind, Peasant};
use crate::pets::CastleHound;
use crate::world::{spawn_ground_log, LogAssets};
use bevy::prelude::*;
use std::f32::consts::TAU;

pub const WARNING_BELL_POS: Vec3 = Vec3::new(3.6, 0.0, -0.8);

const BANDIT_TAUNTS: &[&str] = &[
    "\"Hand over your finest turnips or we shall taunt you a second time!\"",
    "\"Your palisade smells of elderberries, and your High Lord is a hamster!\"",
    "\"We are the Bandits Who Say TURNIP! Demand: 1 bowl of stew and a small shrubbery!\"",
    "\"Tremble before our mighty iron saucepans and slightly bent wooden spoons!\"",
];

const BANDIT_NAMES: &[&str] = &[
    "Sir Robin the Not-Quite-So-Brave",
    "Barnaby's Uninvited Cousin",
    "Dennis the Turnip Rustler",
    "Patsy of the Coconut Brigade",
];

pub struct RaidersPlugin;

impl Plugin for RaidersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkirmishState>()
            .add_systems(Startup, setup_homestead_warning_bell)
            .add_systems(
                Update,
                (
                    handle_blow_challenge_horn_hotkey,
                    update_turnip_bandits_and_defense,
                    animate_tossed_turnip_projectiles,
                    animate_warning_bell_ring,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct SkirmishState {
    pub active_bandits: u32,
    pub bell_ring_timer: f32,
    pub total_skirmishes_won: u32,
}

#[derive(Component)]
pub struct WarningBellClapper;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BanditState {
    WaddlingToHomestead,
    SpinningAndFleeing,
}

#[derive(Component)]
pub struct TurnipBandit {
    pub name: &'static str,
    pub state: BanditState,
    pub bonks_needed: i32,
    pub bonk_cooldown: f32,
    pub waddle_phase: f32,
    pub flee_target: Vec3,
    pub target_pos: Vec3,
}

#[derive(Component)]
pub struct TossedTurnipProjectile {
    pub start: Vec3,
    pub end: Vec3,
    pub progress: f32,
}

#[derive(Resource)]
pub struct BanditAssets {
    pub body_mesh: Handle<Mesh>,
    pub head_mesh: Handle<Mesh>,
    pub pot_hat_mesh: Handle<Mesh>,
    pub club_mesh: Handle<Mesh>,
    pub turnip_proj_mesh: Handle<Mesh>,
    pub rag_mat: Handle<StandardMaterial>,
    pub skin_mat: Handle<StandardMaterial>,
    pub pot_mat: Handle<StandardMaterial>,
    pub club_mat: Handle<StandardMaterial>,
    pub turnip_mat: Handle<StandardMaterial>,
}

fn setup_homestead_warning_bell(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let timber_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.44, 0.30, 0.17), perceptual_roughness: 0.88, ..default() });
    let bronze_bell_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.95, 0.78, 0.24), metallic: 0.82, perceptual_roughness: 0.28, ..default() });

    commands.spawn((Transform::from_translation(WARNING_BELL_POS), Visibility::default())).with_children(|bell| {
        for sx in [-0.45_f32, 0.45_f32] {
            bell.spawn((ExteriorShell, Mesh3d(meshes.add(Cylinder::new(0.10, 2.2))), MeshMaterial3d(timber_mat.clone()), Transform::from_xyz(sx, 1.1, 0.0)));
        }
        bell.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(1.2, 0.16, 0.22))), MeshMaterial3d(timber_mat), Transform::from_xyz(0.0, 2.22, 0.0)));
        bell.spawn((ExteriorShell, WarningBellClapper, Mesh3d(meshes.add(Cone { radius: 0.28, height: 0.45 })), MeshMaterial3d(bronze_bell_mat), Transform::from_xyz(0.0, 1.92, 0.0)));
    });

    commands.insert_resource(BanditAssets {
        body_mesh: meshes.add(Capsule3d::new(0.25, 0.52)),
        head_mesh: meshes.add(Sphere::new(0.20)),
        pot_hat_mesh: meshes.add(Cylinder::new(0.24, 0.24)),
        club_mesh: meshes.add(Cylinder::new(0.06, 0.65)),
        turnip_proj_mesh: meshes.add(Sphere::new(0.15)),
        rag_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.52, 0.26, 0.36), perceptual_roughness: 0.9, ..default() }),
        skin_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.90, 0.74, 0.62), ..default() }),
        pot_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.26, 0.28, 0.32), metallic: 0.85, perceptual_roughness: 0.35, ..default() }),
        club_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.48, 0.32, 0.18), ..default() }),
        turnip_mat: materials.add(StandardMaterial { base_color: Color::srgb(0.78, 0.42, 0.88), emissive: LinearRgba::new(0.25, 0.08, 0.30, 1.0), ..default() }),
    });
}

pub fn spawn_slapstick_bandit_wave(
    commands: &mut Commands,
    assets: &BanditAssets,
    skirmish: &mut SkirmishState,
    feed: &mut ActivityFeed,
) {
    skirmish.bell_ring_timer = 5.0;
    let taunt = BANDIT_TAUNTS[(skirmish.total_skirmishes_won as usize) % BANDIT_TAUNTS.len()];
    feed.push(format!("CHALLENGE HORN BLOWN [B]! Homestead Warning Bell rings as 4 Saucepan-Helmet Turnip Bandits waddle in shouting: {taunt}"));

    for i in 0..4 {
        let angle = 0.6 + (i as f32 * 0.42);
        let start = Vec3::new(angle.cos() * 19.5, 0.0, angle.sin() * 19.5);
        let goal = Vec3::new((i as f32 - 1.5) * 2.0, 0.0, 6.2);
        let name = BANDIT_NAMES[i % BANDIT_NAMES.len()];
        skirmish.active_bandits += 1;

        commands.spawn((
            TemporaryProp,
            TurnipBandit {
                name, state: BanditState::WaddlingToHomestead, bonks_needed: 3, bonk_cooldown: 0.4,
                waddle_phase: i as f32 * 1.4, flee_target: start * 1.25, target_pos: goal,
            },
            Transform::from_translation(start),
            Visibility::default(),
        )).with_children(|b| {
            b.spawn((TemporaryProp, Mesh3d(assets.body_mesh.clone()), MeshMaterial3d(assets.rag_mat.clone()), Transform::from_xyz(0.0, 0.52, 0.0)));
            b.spawn((TemporaryProp, Mesh3d(assets.head_mesh.clone()), MeshMaterial3d(assets.skin_mat.clone()), Transform::from_xyz(0.0, 1.05, 0.0)));
            b.spawn((TemporaryProp, Mesh3d(assets.pot_hat_mesh.clone()), MeshMaterial3d(assets.pot_mat.clone()), Transform::from_xyz(0.0, 1.26, 0.0).with_rotation(Quat::from_rotation_z(0.14))));
            b.spawn((TemporaryProp, Mesh3d(assets.club_mesh.clone()), MeshMaterial3d(assets.club_mat.clone()), Transform::from_xyz(0.28, 0.72, 0.25).with_rotation(Quat::from_rotation_x(0.65))));
        });
    }
}

fn handle_blow_challenge_horn_hotkey(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    assets: Option<Res<BanditAssets>>,
    mut skirmish: ResMut<SkirmishState>,
    mut feed: ResMut<ActivityFeed>,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        if let Some(assets) = assets {
            spawn_slapstick_bandit_wave(&mut commands, &assets, &mut skirmish, &mut feed);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_turnip_bandits_and_defense(
    mut commands: Commands,
    time: Res<Time>,
    assets: Option<Res<BanditAssets>>,
    log_assets: Option<Res<LogAssets>>,
    mut skirmish: ResMut<SkirmishState>,
    mut guilds: ResMut<HomesteadAutomatedGuilds>,
    mut feed: ResMut<ActivityFeed>,
    mut lords_q: Query<(&mut LordMind, &Transform), Without<TurnipBandit>>,
    mut peasants_q: Query<(&mut Peasant, &mut CozyNeeds, &Transform), Without<TurnipBandit>>,
    mut hound_q: Query<(&mut CastleHound, &Transform), Without<TurnipBandit>>,
    mut bandits_q: Query<(Entity, &mut TurnipBandit, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let (Some(assets), Some(log_assets)) = (assets, log_assets) else { return };

    for (b_ent, mut bandit, mut b_tf) in &mut bandits_q {
        bandit.waddle_phase += dt * 12.0;
        bandit.bonk_cooldown = (bandit.bonk_cooldown - dt).max(0.0);
        let b_pos = Vec3::new(b_tf.translation.x, 0.0, b_tf.translation.z);

        match bandit.state {
            BanditState::WaddlingToHomestead => {
                for (mut lord, _) in &mut lords_q { lord.target_pos = b_pos; lord.urgency_timer = 2.5; }
                let delta = bandit.target_pos - b_pos;
                if delta.length() > 0.6 {
                    let dir = delta.normalize();
                    let next = b_pos + dir * 2.6 * dt;
                    b_tf.translation = Vec3::new(next.x, bandit.waddle_phase.sin().abs() * 0.14, next.z);
                    b_tf.look_to(dir, Vec3::Y);
                }

                if bandit.bonk_cooldown <= 0.0 {
                    let mut defender_pos = lords_q.iter().find(|(_, t)| t.translation.distance(b_pos) <= 5.5).map(|(_, t)| t.translation)
                        .or_else(|| peasants_q.iter().find(|(_, _, t)| t.translation.distance(b_pos) <= 8.5).map(|(_, _, t)| t.translation));
                    if defender_pos.is_none() {
                        if let Ok((mut hound, h_tf)) = hound_q.get_single_mut() {
                            if h_tf.translation.distance(b_pos) <= 6.5 { hound.jump_timer = 0.55; defender_pos = Some(h_tf.translation); }
                        }
                    }

                    if let Some(from_pos) = defender_pos {
                        bandit.bonk_cooldown = 0.85;
                        bandit.bonks_needed -= 1;
                        commands.spawn((
                            TemporaryProp,
                            TossedTurnipProjectile { start: from_pos + Vec3::Y * 1.1, end: b_pos + Vec3::Y * 1.1, progress: 0.0 },
                            Mesh3d(assets.turnip_proj_mesh.clone()), MeshMaterial3d(assets.turnip_mat.clone()),
                            Transform::from_translation(from_pos + Vec3::Y * 1.1),
                        ));
                        if bandit.bonks_needed <= 0 {
                            bandit.state = BanditState::SpinningAndFleeing;
                            spawn_ground_log(&mut commands, &log_assets, b_pos, 0.8);
                            guilds.turnips_stored += 3;
                            feed.push(format!("*CLANG!* {} got pelted on his saucepan helmet, dropped 1 Timber Log + 3 Turnips, and yelled: \"RUN AWAY! RUN AWAY!\"", bandit.name));
                            for (mut p, mut cozy, _) in &mut peasants_q {
                                p.hop_timer = 0.50; cozy.stew_buff_timer = 30.0; cozy.energy = 100.0;
                            }
                        }
                    }
                }
            }
            BanditState::SpinningAndFleeing => {
                let delta = bandit.flee_target - b_pos;
                if delta.length() > 1.0 {
                    let dir = delta.normalize();
                    let next = b_pos + dir * 6.2 * dt;
                    b_tf.translation = Vec3::new(next.x, (bandit.waddle_phase * 2.0).sin().abs() * 0.25, next.z);
                    b_tf.rotation = Quat::from_rotation_y((bandit.waddle_phase * 1.8) % TAU);
                } else {
                    commands.entity(b_ent).despawn_recursive();
                    skirmish.active_bandits = skirmish.active_bandits.saturating_sub(1);
                    if skirmish.active_bandits == 0 {
                        skirmish.total_skirmishes_won += 1;
                        feed.push("SLAPSTICK VICTORY! All Turnip Bandits fled into the woods leaving free logs & turnips! Village cheered (+50% Speed)!".to_string());
                    }
                }
            }
        }
    }
}

fn animate_tossed_turnip_projectiles(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut TossedTurnipProjectile, &mut Transform)>) {
    let dt = time.delta_secs();
    for (ent, mut proj, mut tf) in &mut query {
        proj.progress += dt * 3.2;
        let t = proj.progress.min(1.0);
        tf.translation = proj.start.lerp(proj.end, t) + Vec3::Y * ((t * std::f32::consts::PI).sin() * 1.8);
        if t >= 1.0 { commands.entity(ent).despawn_recursive(); }
    }
}

fn animate_warning_bell_ring(time: Res<Time>, mut skirmish: ResMut<SkirmishState>, mut bell_q: Query<&mut Transform, With<WarningBellClapper>>) {
    if skirmish.bell_ring_timer > 0.0 {
        skirmish.bell_ring_timer = (skirmish.bell_ring_timer - time.delta_secs()).max(0.0);
        let swing = (time.elapsed_secs() * 18.0).sin() * 0.45;
        for mut tf in &mut bell_q { tf.rotation = Quat::from_rotation_z(swing); }
    } else {
        for mut tf in &mut bell_q { tf.rotation = Quat::IDENTITY; }
    }
}

