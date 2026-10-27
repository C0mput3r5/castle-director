pub mod shell_mesh;
pub use shell_mesh::*;

use crate::economy::{STEW_POT_POS, TURNIP_PATCH_POS};
use crate::npc::{ActivityFeed, LordMind, OverseerRole, Peasant, SpecialistOverseer};
use crate::raiders::WARNING_BELL_POS;
use crate::state::{HistoricalEraState, HoverTooltipState, SimulationPace, VetoedZones};
use crate::world::{spawn_ground_log, LogAssets, FARMHOUSE_POS, STOCKPILE_POS};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::f32::consts::FRAC_PI_2;

pub const STAKE_SPACING: f32 = 1.5;

pub struct DirectorBlueprintPlugin;
impl Plugin for DirectorBlueprintPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirectorMode>().init_resource::<GameSpeed>().init_resource::<BlueprintStrokeState>()
            .add_systems(Update, (handle_mode_speed_and_tower_keys, smart_brush_and_priority_bell_system, update_inspect_hover_tooltip, draw_blueprint_gizmos));
    }
}

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum DirectorMode { #[default] Inspect, DrawPalisade }

#[derive(Resource)]
pub struct GameSpeed { pub multiplier: f32 }
impl Default for GameSpeed { fn default() -> Self { Self { multiplier: 1.0 } } }

#[derive(Resource, Default)]
pub struct BlueprintStrokeState {
    pub current_stroke_id: u32, pub last_stake_pos: Option<Vec3>, pub stroke_points: Vec<Vec3>,
    pub cursor_ground_pos: Option<Vec3>, pub hovering_tower_handle: bool, pub whim_reaction_cooldown: f32,
}

#[allow(clippy::too_many_arguments)]
fn handle_mode_speed_and_tower_keys(
    time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mut mouse_wheel: EventReader<MouseWheel>,
    mut mode: ResMut<DirectorMode>, mut pace: ResMut<SimulationPace>, mut stroke: ResMut<BlueprintStrokeState>,
    mut feed: ResMut<ActivityFeed>, mut towers: Query<&mut TowerFoundation>, mut sites: Query<&mut ConstructionSite>,
) {
    stroke.whim_reaction_cooldown = (stroke.whim_reaction_cooldown - time.delta_secs()).max(0.0);
    if keys.just_pressed(KeyCode::Digit1) { *mode = DirectorMode::Inspect; }
    if keys.just_pressed(KeyCode::Digit2) { *mode = DirectorMode::DrawPalisade; }
    if keys.just_pressed(KeyCode::Space) {
        *pace = match *pace {
            SimulationPace::PreWorkBriefing => { feed.push("WORK HORN SOUNDED [Space]! High Lord Aldric, Mason Cedric & the Peasants begin building the castle!"); SimulationPace::Running }
            SimulationPace::Running => { feed.push("WORK PAUSED [Space]: Servants stand by peacefully while you inspect or nudge plans."); SimulationPace::Paused }
            SimulationPace::Paused => { feed.push("WORK RESUMED [Space]: Servants return to their duties!"); SimulationPace::Running }
        };
    }
    let cursor_pt = stroke.cursor_ground_pos;
    stroke.hovering_tower_handle = cursor_pt.map_or(false, |pt| towers.iter().any(|t| t.center.distance(pt) <= (t.radius + 1.6)));
    let mut story_delta: i32 = 0;
    if keys.just_pressed(KeyCode::PageUp) { story_delta = 1; }
    else if keys.just_pressed(KeyCode::PageDown) { story_delta = -1; }
    else if stroke.hovering_tower_handle {
        for ev in mouse_wheel.read() { if ev.y > 0.1 { story_delta = 1; } else if ev.y < -0.1 { story_delta = -1; } }
    }
    if story_delta != 0 {
        if let Some(mut tower) = towers.iter_mut().find(|t| cursor_pt.map_or(true, |pt| t.center.distance(pt) <= (t.radius + 3.0))) {
            let new_stories = (tower.stories as i32 + story_delta).clamp(1, 4) as u32;
            if new_stories != tower.stories {
                tower.stories = new_stories; let sid = tower.stroke_id;
                for mut site in &mut sites { if site.stroke_id == sid { site.kind = SiteKind::TowerKeepWall { stories: new_stories }; } }
                feed.push(format!("SUBCONSCIOUS WHIM: Mason Cedric nods approvingly: \"Making that Keep {new_stories} stories tall was my finest idea yet!\""));
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn smart_brush_and_priority_bell_system(
    mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mouse_buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<DirectorMode>,
    asset_pack: (Option<Res<BlueprintAssets>>, Option<Res<LogAssets>>, Res<HistoricalEraState>, ResMut<VetoedZones>),
    mut stats: ResMut<BlueprintStats>, mut stroke: ResMut<BlueprintStrokeState>, mut feed: ResMut<ActivityFeed>,
    window_q: Query<&Window, With<PrimaryWindow>>, camera_q: Query<(&Camera, &GlobalTransform)>,
    ui_interaction_q: Query<&Interaction, With<Button>>, mut sites_q: Query<(Entity, &mut ConstructionSite)>,
    built_q: Query<(Entity, &BuiltStructure)>, mut towers_q: Query<(Entity, &mut TowerFoundation)>,
    mut peasants_q: Query<(&mut Peasant, &Transform)>, mut lords_q: Query<&mut LordMind>,
) {
    let (Some(assets), Some(log_assets), era_state, mut vetoed) = asset_pack else { return };
    let (Ok(window), Ok((camera, cam_tf))) = (window_q.get_single(), camera_q.get_single()) else { return };
    if ui_interaction_q.iter().any(|i| *i != Interaction::None) { return; }
    let Some(cursor_pos) = window.cursor_position() else { stroke.cursor_ground_pos = None; return; };
    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else { return };
    let Some(dist) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else { return };
    let ground_pt = ray.get_point(dist);
    stroke.cursor_ground_pos = Some(ground_pt);

    if *mode == DirectorMode::Inspect {
        stroke.last_stake_pos = None; stroke.stroke_points.clear();
        if mouse_buttons.just_pressed(MouseButton::Left) {
            let mut rang_bell_pos: Option<Vec3> = None;
            for (_, mut site) in &mut sites_q {
                if site.position.distance(ground_pt) <= 1.75 { site.priority_bell = true; rang_bell_pos = Some(site.position); break; }
            }
            if rang_bell_pos.is_none() {
                for (_, mut tower) in &mut towers_q {
                    if tower.center.distance(ground_pt) <= tower.radius + 1.2 {
                        tower.priority_bell = true; let sid = tower.stroke_id;
                        for (_, mut site) in &mut sites_q { if site.stroke_id == sid { site.priority_bell = true; } }
                        rang_bell_pos = Some(tower.center); break;
                    }
                }
            }
            if let Some(bell_pos) = rang_bell_pos {
                for (mut p, p_tf) in &mut peasants_q { if p_tf.translation.distance(bell_pos) < 28.0 { p.hop_timer = 0.55; } }
                feed.push("PRIORITY BELL RUNG [1]: Servants hopped joyfully & prioritized this section (2.5x speed)!");
            }
        }
        return;
    }

    // Right-Click Scrubber in [2]: Erases stake/wall & records in VetoedZones so NPCs NEVER re-stake it!
    if mouse_buttons.pressed(MouseButton::Right) {
        stroke.last_stake_pos = None; stroke.stroke_points.clear();
        for (ent, site) in &sites_q {
            if site.position.distance(ground_pt) <= 1.45 {
                vetoed.record_veto(site.position);
                if let Some(log_vis) = site.delivered_log_vis { commands.entity(log_vis).despawn_recursive(); }
                if let Some(scaff) = site.scaffolding_vis { commands.entity(scaff).despawn_recursive(); }
                if site.has_log { spawn_ground_log(&mut commands, &log_assets, site.position, 0.5); }
                commands.entity(ent).despawn_recursive();
                stats.stakes_planned = stats.stakes_planned.saturating_sub(1);
                if stroke.whim_reaction_cooldown <= 0.0 {
                    stroke.whim_reaction_cooldown = 2.5;
                    for mut lord in &mut lords_q {
                        lord.current_thought = "\"On second thought, that spot had terrible feng shui. I shall leave it open as a courtyard!\"".into();
                    }
                    for (mut p, _) in &mut peasants_q {
                        p.status_text = format!("{} nods: \"As the Master wills it! Leaving that courtyard open.\"", p.name);
                    }
                    feed.push("SUBCONSCIOUS VETO [Right-Click]: Lord Aldric muses: \"On second thought, that spot had terrible feng shui. I shall leave it open as a courtyard!\"");
                }
                return;
            }
        }
        for (ent, built) in &built_q {
            if built.position.distance(ground_pt) <= 1.5 {
                vetoed.record_veto(built.position);
                commands.entity(ent).despawn_recursive();
                stats.posts_completed = stats.posts_completed.saturating_sub(1);
                spawn_ground_log(&mut commands, &log_assets, built.position, 1.1);
                for mut lord in &mut lords_q {
                    lord.current_thought = "\"On second thought, that spot had terrible feng shui. I shall leave it open as a courtyard!\"".into();
                }
                feed.push("SUBCONSCIOUS VETO: Dismantled wall & marked zone as an open courtyard (NPCs will never re-stake here)!");
                return;
            }
        }
        for (ent, tower) in &towers_q {
            if tower.center.distance(ground_pt) <= 1.9 {
                vetoed.record_veto(tower.center);
                commands.entity(ent).despawn_recursive();
                return;
            }
        }
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        stroke.current_stroke_id = stroke.current_stroke_id.wrapping_add(1).max(1);
        stroke.stroke_points.clear();
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        let sid = stroke.current_stroke_id;
        vetoed.positions.retain(|v| v.distance(ground_pt) > 2.0);

        // 1. In-Place Era 3+ Retrofitting: Dragging [2] over an existing wooden wall upgrades it to Norman Stone in-place!
        if era_state.current.uses_stone_masonry() {
            for (ent, built) in &built_q {
                if !built.is_stone && built.position.distance(ground_pt) <= 1.35 {
                    retrofit_timber_structure_to_stone(&mut commands, &mut meshes, &assets, ent, built, era_state.current);
                    feed.push("IN-PLACE RETROFIT [2]: Master Mason Cedric upgraded a timber palisade segment into Norman Ashlar Stone!");
                    return;
                }
            }
        }

        // 2. In-Place Gatehouse Carving: Dragging [2] across an existing non-gate stake (like Sir Barnaby's gateless wall!) converts it in-place!
        for (_, mut site) in &mut sites_q {
            if site.stroke_id != sid && !matches!(site.kind, SiteKind::GatehouseArchway) && site.position.distance(ground_pt) <= 1.25 {
                site.kind = SiteKind::GatehouseArchway;
                site.priority_bell = true;
                for mut lord in &mut lords_q {
                    lord.current_thought = "\"Wait—did I sketch that gatehouse archway there? Brilliant stroke of genius on my part!\"".into();
                }
                feed.push("SUBCONSCIOUS WHIM [2]: Carved a Gatehouse Archway in-place! Servants hauling logs seamlessly continue to it!");
                stroke.last_stake_pos = Some(site.position);
                stroke.stroke_points.push(site.position);
                return;
            }
        }

        match stroke.last_stake_pos {
            None => {
                if !sites_q.iter().any(|(_, s)| s.position.distance(ground_pt) < 0.7) {
                    let kind = detect_gate_crossing(ground_pt, sid, &sites_q, &built_q);
                    spawn_stake_site(&mut commands, &mut meshes, &assets, ground_pt, None, sid, kind);
                    stats.stakes_planned += 1; stroke.last_stake_pos = Some(ground_pt); stroke.stroke_points.push(ground_pt);
                    if stroke.whim_reaction_cooldown <= 0.0 {
                        stroke.whim_reaction_cooldown = 3.5;
                        feed.push("UNSEEN WHIM: Master Mason Cedric beams: \"Hmm, looking at the stakes again, shifting the wall here makes much better sense. Carry on, lads!\"");
                    }
                }
            }
            Some(mut last_pos) => {
                let mut diff = Vec3::new(ground_pt.x - last_pos.x, 0.0, ground_pt.z - last_pos.z);
                while diff.length() >= STAKE_SPACING {
                    let next_pos = last_pos + diff.normalize() * STAKE_SPACING;
                    let kind = detect_gate_crossing(next_pos, sid, &sites_q, &built_q);
                    if matches!(kind, SiteKind::GatehouseArchway) {
                        for mut lord in &mut lords_q { lord.current_thought = "\"Wait—did I sketch that gatehouse there? Brilliant stroke of genius on my part!\"".into(); }
                        feed.push("UNSEEN WHIM: High Lord Aldric gasps: \"Wait—did I sketch that gatehouse there? Brilliant stroke of genius on my part!\"");
                    }
                    let spacing = if matches!(kind, SiteKind::GatehouseArchway) { STAKE_SPACING * 1.35 } else { STAKE_SPACING };
                    let placed_pos = last_pos + diff.normalize() * spacing;
                    spawn_stake_site(&mut commands, &mut meshes, &assets, placed_pos, Some(last_pos), sid, kind);
                    stats.stakes_planned += 1; last_pos = placed_pos; stroke.last_stake_pos = Some(last_pos); stroke.stroke_points.push(last_pos);
                    diff = Vec3::new(ground_pt.x - last_pos.x, 0.0, ground_pt.z - last_pos.z);
                }
            }
        }
    } else if mouse_buttons.just_released(MouseButton::Left) {
        if stroke.stroke_points.len() >= 4 {
            let first = stroke.stroke_points[0];
            let last = *stroke.stroke_points.last().unwrap();
            if first.distance(last) <= 2.5 {
                let center = stroke.stroke_points.iter().copied().sum::<Vec3>() / (stroke.stroke_points.len() as f32);
                let radius = stroke.stroke_points.iter().map(|p| p.distance(center)).sum::<f32>() / (stroke.stroke_points.len() as f32);
                let sid = stroke.current_stroke_id;
                for (_, mut site) in &mut sites_q { if site.stroke_id == sid { site.kind = SiteKind::TowerKeepWall { stories: 2 }; } }
                commands.spawn((TowerFoundation { center, radius: radius.max(1.8), stories: 2, stroke_id: sid, priority_bell: false }, Transform::from_translation(center), Visibility::default()));
                feed.push("UNSEEN WHIM: Sir Barnaby cheers: \"Aha! A 2-Story Watchtower loop—I knew my tactical instincts were unmatched!\"");
            }
        }
        stroke.last_stake_pos = None; stroke.stroke_points.clear();
    }
}

#[allow(clippy::too_many_arguments)]
fn update_inspect_hover_tooltip(
    mode: Res<DirectorMode>, stroke: Res<BlueprintStrokeState>, mut tooltip: ResMut<HoverTooltipState>,
    lords_q: Query<(&LordMind, &Transform)>, overseers_q: Query<(&SpecialistOverseer, &Transform)>,
    peasants_q: Query<(&Peasant, &Transform)>, sites_q: Query<&ConstructionSite>, built_q: Query<&BuiltStructure>,
) {
    if *mode != DirectorMode::Inspect { tooltip.visible = false; return; }
    let Some(pt) = stroke.cursor_ground_pos else { tooltip.visible = false; return; };

    for (lord, tf) in &lords_q {
        if tf.translation.distance(pt) <= 1.8 {
            *tooltip = HoverTooltipState {
                visible: true, title: format!("{} ({})", lord.name, lord.title),
                what_it_does: format!("Autonomously plans castle rings & emits a {:.1}m command speed aura. Thought: {}", lord.aura_radius(), lord.current_thought),
                how_to_interact: "Press [V] to Mind-Meld into his eyes, or Double-Click to frame camera.".into(),
            };
            return;
        }
    }
    for (ov, tf) in &overseers_q {
        if tf.translation.distance(pt) <= 1.8 {
            *tooltip = match ov.role {
                OverseerRole::MasterMasonCedric => HoverTooltipState {
                    visible: true, title: "Master Mason Cedric (Royal Architecture Overseer)".into(),
                    what_it_does: "Autonomously stakes out concentric castle rings, erects scaffolding, and retrofits timber walls to stone in Era 3+.".into(),
                    how_to_interact: "Press [G] to have him propose the next Concentric Ring sector immediately!".into(),
                },
                OverseerRole::RowanTheReeve => HoverTooltipState {
                    visible: true, title: "Rowan the Reeve (Village Logistics Overseer)".into(),
                    what_it_does: "Supervises the Turnip Patch, Stew Pot, and Timber Stockpile crew assignments.".into(),
                    how_to_interact: "Press [R] to cycle the Reeve's Priority Decree.".into(),
                },
            };
            return;
        }
    }
    for (p, tf) in &peasants_q {
        if tf.translation.distance(pt) <= 1.6 {
            *tooltip = HoverTooltipState {
                visible: true, title: format!("Peasant {} (Loyal Homestead Servant)", p.name),
                what_it_does: format!("Fells sustainable trees, hauls logs, digs ditches, and builds walls without question. Status: {}", p.status_text),
                how_to_interact: "Double-Click to follow, or ring a Stake's Priority Bell [Left-Click] to direct them.".into(),
            };
            return;
        }
    }
    if FARMHOUSE_POS.distance(pt) <= 3.0 {
        *tooltip = HoverTooltipState { visible: true, title: "Central Homestead Farmhouse (ExteriorShell)".into(), what_it_does: "Center of the Concentric Castle Master Plan where villagers gather and volunteers arrive.".into(), how_to_interact: "Press [3] to summon an extra volunteer peasant from the village.".into() }; return;
    }
    if STEW_POT_POS.distance(pt) <= 2.2 || STOCKPILE_POS.distance(pt) <= 2.5 || TURNIP_PATCH_POS.distance(pt) <= 2.6 || WARNING_BELL_POS.distance(pt) <= 2.0 {
        *tooltip = HoverTooltipState { visible: true, title: "Homestead Stew Pot, Stockpile & Turnip Patch".into(), what_it_does: "Feeds tired servants (+50% speed buff), stores timber logs, and rings the [B] Slapstick Bandit Horn.".into(), how_to_interact: "Automatic cozy logistics loop (or press [B] for a non-destructive turnip skirmish).".into() }; return;
    }
    if let Some(site) = sites_q.iter().find(|s| s.position.distance(pt) <= 1.5) {
        *tooltip = HoverTooltipState { visible: true, title: format!("Staked Blueprint ({:?})", site.kind), what_it_does: "Surveyed stake waiting for a servant to haul 1 Timber Log, dig the ditch, and erect the wall.".into(), how_to_interact: "[Left-Click] Ring Priority Bell (2.5x speed) | [2] Drag across to carve Gatehouse or [Right-Click] Veto.".into() }; return;
    }
    if let Some(built) = built_q.iter().find(|b| b.position.distance(pt) <= 1.6) {
        let mat_tag = if built.is_stone { "Norman Ashlar Stone" } else { "Timber Palisade" };
        *tooltip = HoverTooltipState { visible: true, title: format!("Completed {:?} ({mat_tag} — ExteriorShell)", built.kind), what_it_does: "Watertight 3D solid fortification ready for [F12] .STL 3D printing.".into(), how_to_interact: "In Era 3+, Mason Cedric (or dragging [2]) upgrades Timber to Stone in-place! [Right-Click] in [2] to Veto.".into() }; return;
    }
    tooltip.visible = false;
}

fn detect_gate_crossing(pos: Vec3, current_stroke_id: u32, sites_q: &Query<(Entity, &mut ConstructionSite)>, built_q: &Query<(Entity, &BuiltStructure)>) -> SiteKind {
    let crosses_stake = sites_q.iter().any(|(_, s)| s.stroke_id != current_stroke_id && s.position.distance(pos) <= 1.25);
    let crosses_built = built_q.iter().any(|(_, b)| b.stroke_id != current_stroke_id && b.position.distance(pos) <= 1.35);
    if crosses_stake || crosses_built { SiteKind::GatehouseArchway } else { SiteKind::PalisadePost }
}

fn draw_blueprint_gizmos(mut gizmos: Gizmos, mode: Res<DirectorMode>, stroke: Res<BlueprintStrokeState>, sites: Query<&ConstructionSite>) {
    for site in &sites {
        let color = if site.has_log { Color::srgb(0.25, 0.95, 0.45) } else { Color::srgb(0.98, 0.85, 0.35) };
        gizmos.circle(Isometry3d::new(site.position + Vec3::Y * 0.05, Quat::from_rotation_x(FRAC_PI_2)), 0.35, color);
        if let Some(prev) = site.prev_stake_pos {
            gizmos.line(prev + Vec3::Y * 0.58, site.position + Vec3::Y * 0.58, Color::srgb(0.96, 0.88, 0.52));
        }
    }
    if *mode == DirectorMode::DrawPalisade {
        if let Some(pt) = stroke.cursor_ground_pos {
            gizmos.circle(Isometry3d::new(pt + Vec3::Y * 0.06, Quat::from_rotation_x(FRAC_PI_2)), 0.48, Color::srgb(1.0, 0.92, 0.35));
            if let Some(last) = stroke.last_stake_pos {
                let ribbon_col = Color::srgb(0.45, 0.98, 0.72);
                gizmos.line(last + Vec3::Y * 0.12, pt + Vec3::Y * 0.12, ribbon_col);
                let dir = (pt - last).normalize_or_zero();
                let side = Vec3::new(-dir.z, 0.0, dir.x) * 0.25;
                gizmos.line(last + side + Vec3::Y * 0.08, pt + side + Vec3::Y * 0.08, ribbon_col);
                gizmos.line(last - side + Vec3::Y * 0.08, pt - side + Vec3::Y * 0.08, ribbon_col);
            }
        }
    }
}
