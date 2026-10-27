use crate::blueprint::{
    retrofit_timber_structure_to_stone, spawn_ditch_and_scaffolding_stage, BlueprintAssets,
    BuiltStructure, ConstructionSite, TemporaryProp,
};
use crate::economy::{STEW_POT_POS, TURNIP_PATCH_POS};
use crate::npc::{ActivityFeed, PeasantRoster};
use crate::state::HistoricalEraState;
use crate::world::STOCKPILE_POS;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReeveDecree {
    #[default]
    BalancedVillage,
    AllHandsTimberAndQuarrying,
    AllHandsCastleConstruction,
    FestivalAndFeastRest,
}

impl ReeveDecree {
    pub fn next(self) -> Self {
        match self {
            Self::BalancedVillage => Self::AllHandsTimberAndQuarrying,
            Self::AllHandsTimberAndQuarrying => Self::AllHandsCastleConstruction,
            Self::AllHandsCastleConstruction => Self::FestivalAndFeastRest,
            Self::FestivalAndFeastRest => Self::BalancedVillage,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BalancedVillage => "Balanced Village",
            Self::AllHandsTimberAndQuarrying => "All-Hands Timber & Quarrying",
            Self::AllHandsCastleConstruction => "All-Hands Castle Construction",
            Self::FestivalAndFeastRest => {
                "Festival & Feast Rest (Everyone gathers at campfire for 2x morale)"
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OverseerRole {
    RowanTheReeve,
    MasterMasonCedric,
}

#[derive(Component)]
pub struct SpecialistOverseer {
    pub role: OverseerRole,
    pub target_pos: Vec3,
    pub pause_timer: f32,
    pub walk_phase: f32,
    pub waypoint_idx: usize,
    pub retrofit_cooldown: f32,
}

pub fn spawn_specialist_overseers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    roster: &PeasantRoster,
) {
    let reeve_green_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.56, 0.28),
        perceptual_roughness: 0.7,
        ..default()
    });
    let reeve_hat_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.22, 0.12),
        perceptual_roughness: 0.85,
        ..default()
    });
    let mason_blue_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.42, 0.78),
        perceptual_roughness: 0.65,
        ..default()
    });
    let apron_white_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.90, 0.85),
        perceptual_roughness: 0.85,
        ..default()
    });
    let board_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.84, 0.72, 0.48),
        perceptual_roughness: 0.8,
        ..default()
    });

    // 1. Rowan the Reeve (Economy & Logistics Overseer)
    commands
        .spawn((
            TemporaryProp,
            SpecialistOverseer {
                role: OverseerRole::RowanTheReeve,
                target_pos: TURNIP_PATCH_POS,
                pause_timer: 1.5,
                walk_phase: 0.0,
                waypoint_idx: 0,
                retrofit_cooldown: 0.0,
            },
            Transform::from_xyz(-3.5, 0.0, -2.0),
            Visibility::default(),
        ))
        .with_children(|p| {
            p.spawn((
                TemporaryProp,
                Mesh3d(roster.npc_mesh.clone()),
                MeshMaterial3d(reeve_green_mat),
                Transform::from_xyz(0.0, 0.54, 0.0),
            ));
            p.spawn((
                TemporaryProp,
                Mesh3d(roster.head_mesh.clone()),
                MeshMaterial3d(roster.skin_mat.clone()),
                Transform::from_xyz(0.0, 1.10, 0.0),
            ));
            // Distinct wide-brimmed Reeve hat
            p.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cylinder::new(0.32, 0.06))),
                MeshMaterial3d(reeve_hat_mat.clone()),
                Transform::from_xyz(0.0, 1.28, 0.0),
            ));
            p.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cylinder::new(0.18, 0.20))),
                MeshMaterial3d(reeve_hat_mat),
                Transform::from_xyz(0.0, 1.38, 0.0),
            ));
        });

    // 2. Master Mason Cedric (Architecture Overseer holding a drafting board)
    commands
        .spawn((
            TemporaryProp,
            SpecialistOverseer {
                role: OverseerRole::MasterMasonCedric,
                target_pos: Vec3::new(0.0, 0.0, 5.2),
                pause_timer: 1.2,
                walk_phase: 0.0,
                waypoint_idx: 0,
                retrofit_cooldown: 0.0,
            },
            Transform::from_xyz(1.2, 0.0, 4.4),
            Visibility::default(),
        ))
        .with_children(|p| {
            p.spawn((
                TemporaryProp,
                Mesh3d(roster.npc_mesh.clone()),
                MeshMaterial3d(mason_blue_mat.clone()),
                Transform::from_xyz(0.0, 0.54, 0.0),
            ));
            // Mason's Guild Apron
            p.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cuboid::new(0.36, 0.52, 0.08))),
                MeshMaterial3d(apron_white_mat),
                Transform::from_xyz(0.0, 0.48, 0.22),
            ));
            p.spawn((
                TemporaryProp,
                Mesh3d(roster.head_mesh.clone()),
                MeshMaterial3d(roster.skin_mat.clone()),
                Transform::from_xyz(0.0, 1.10, 0.0),
            ));
            // Master Mason Cap
            p.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cuboid::new(0.30, 0.14, 0.30))),
                MeshMaterial3d(mason_blue_mat),
                Transform::from_xyz(0.0, 1.30, 0.0),
            ));
            // Drafting Board held in front
            p.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cuboid::new(0.42, 0.04, 0.32))),
                MeshMaterial3d(board_mat),
                Transform::from_xyz(0.18, 0.72, 0.34).with_rotation(Quat::from_rotation_x(0.35)),
            ));
        });
}

pub fn handle_reeve_decree_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    mut decree: ResMut<ReeveDecree>,
    mut feed: ResMut<ActivityFeed>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        *decree = decree.next();
        feed.push(format!(
            "REEVE'S PRIORITY DECREE [R]: Switched village order to '{}'!",
            decree.label()
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_specialist_overseers_ai(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    bp_assets: Option<Res<BlueprintAssets>>,
    era_state: Res<HistoricalEraState>,
    decree: Res<ReeveDecree>,
    mut feed: ResMut<ActivityFeed>,
    mut gizmos: Gizmos,
    mut sites_q: Query<&mut ConstructionSite>,
    built_q: Query<(Entity, &BuiltStructure)>,
    mut overseers_q: Query<(&mut SpecialistOverseer, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let site_positions: Vec<Vec3> = sites_q.iter().map(|s| s.position).collect();
    let timber_built_positions: Vec<Vec3> = built_q
        .iter()
        .filter(|(_, b)| !b.is_stone)
        .map(|(_, b)| b.position)
        .collect();

    for (mut ov, mut tf) in &mut overseers_q {
        ov.retrofit_cooldown = (ov.retrofit_cooldown - dt).max(0.0);
        let pos = Vec3::new(tf.translation.x, 0.04, tf.translation.z);
        let aura_col = match ov.role {
            OverseerRole::RowanTheReeve => Color::srgb(0.28, 0.92, 0.48),
            OverseerRole::MasterMasonCedric => Color::srgb(0.32, 0.78, 1.0),
        };
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(FRAC_PI_2)),
            4.5,
            aura_col,
        );

        // Master Mason Cedric automatically summons scaffolding & trenches on nearby blueprint stakes,
        // and in Era 3+ retrofits existing Timber Palisades in-place into Norman Ashlar Stone Walls!
        if ov.role == OverseerRole::MasterMasonCedric {
            if let Some(ref assets) = bp_assets {
                for mut site in &mut sites_q {
                    if !site.ditch_dug && site.position.distance(pos) <= 4.2 {
                        spawn_ditch_and_scaffolding_stage(
                            &mut commands,
                            &mut meshes,
                            assets,
                            &mut site,
                            era_state.current,
                        );
                    }
                }
                if era_state.current.uses_stone_masonry() && ov.retrofit_cooldown <= 0.0 {
                    if let Some((ent, built)) = built_q
                        .iter()
                        .find(|(_, b)| !b.is_stone && b.position.distance(pos) <= 4.8)
                    {
                        ov.retrofit_cooldown = 3.2;
                        retrofit_timber_structure_to_stone(
                            &mut commands,
                            &mut meshes,
                            assets,
                            ent,
                            built,
                            era_state.current,
                        );
                        feed.push("ERA RETROFIT: Master Mason Cedric upgraded an old timber palisade segment in-place into Norman Ashlar Stone!");
                    }
                }
            }
        }

        if *decree == ReeveDecree::FestivalAndFeastRest {
            ov.target_pos = STEW_POT_POS + Vec3::new(1.2, 0.0, -1.0);
        }

        let delta = Vec3::new(ov.target_pos.x - pos.x, 0.0, ov.target_pos.z - pos.z);
        if delta.length() > 0.45 {
            let dir = delta.normalize();
            let next = Vec3::new(pos.x, 0.0, pos.z) + dir * 2.9 * dt;
            ov.walk_phase += dt * 9.5;
            tf.translation = Vec3::new(next.x, ov.walk_phase.sin().abs() * 0.09, next.z);
            tf.look_to(dir, Vec3::Y);
        } else {
            tf.translation.y = 0.0;
            ov.pause_timer -= dt;
            if ov.pause_timer <= 0.0 {
                ov.pause_timer = 3.2;
                ov.waypoint_idx = ov.waypoint_idx.wrapping_add(1);
                match ov.role {
                    OverseerRole::RowanTheReeve => {
                        let waypoints = [
                            TURNIP_PATCH_POS + Vec3::new(0.0, 0.0, 1.0),
                            STOCKPILE_POS + Vec3::new(-1.2, 0.0, 0.8),
                            STEW_POT_POS + Vec3::new(1.2, 0.0, 0.5),
                            Vec3::new(-9.0, 0.0, -7.0),
                        ];
                        ov.target_pos = waypoints[ov.waypoint_idx % waypoints.len()];
                    }
                    OverseerRole::MasterMasonCedric => {
                        if !site_positions.is_empty() {
                            ov.target_pos = site_positions[ov.waypoint_idx % site_positions.len()]
                                + Vec3::new(0.0, 0.0, -1.1);
                        } else if era_state.current.uses_stone_masonry() && !timber_built_positions.is_empty() {
                            ov.target_pos = timber_built_positions[ov.waypoint_idx % timber_built_positions.len()]
                                + Vec3::new(0.0, 0.0, -1.1);
                        } else {
                            ov.target_pos = Vec3::new(0.0, 0.0, 4.8);
                        }
                    }
                }
            }
        }
    }
}
