use crate::blueprint::{
    spawn_stake_site, BlueprintAssets, BlueprintStats, BlueprintStrokeState, BuiltStructure,
    ConstructionSite, SiteKind, TowerFoundation,
};
use crate::npc::{
    ActiveFeudalDebate, ActivityFeed, LordArchetype, LordMind, OverseerRole, SpecialistOverseer,
};
use crate::state::{HistoricalEraState, SimulationPace, VetoedZones};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub const INNER_RING_RADIUS: f32 = 11.0;
pub const MIDDLE_RING_RADIUS: f32 = 20.0;
pub const OUTER_RING_RADIUS: f32 = 29.0;

#[derive(Resource)]
pub struct AutonomousArchitectState {
    pub phase_index: u32,
    pub countdown: f32,
    pub propose_now_requested: bool,
}

impl Default for AutonomousArchitectState {
    fn default() -> Self {
        Self {
            phase_index: 1,
            countdown: 13.0,
            propose_now_requested: false,
        }
    }
}

impl AutonomousArchitectState {
    pub fn next_phase_description(&self) -> &'static str {
        match self.phase_index {
            1 => "Inner Ring (11m): North Homestead Arc + Cart Gate [Lord Aldric]",
            2 => "Inner Ring (11m): East Sector [Feudal Debate: Gatehouse vs. Watchtower!]",
            3 => "Inner Ring (11m): West Turnip Yard Arc + Cart Gate [Lord Aldric]",
            4 => "Middle Ring (20m): North Curtain [Feudal Debate: Aldric vs. Barnaby!]",
            5 => "Middle Ring (20m): South Outer Bailey Arc + Gatehouse [Lord Aldric]",
            6 => "Middle Ring (20m): East Curtain [Feudal Debate: Aldric vs. Barnaby!]",
            7 => "Middle Ring (20m): West Curtain Wall + Cart Gate [Lord Aldric]",
            _ => "Outer Ring (29m): Concentric Fortress Curtain & Barbican",
        }
    }
}

struct RingSectorSpec {
    ring_name: &'static str,
    sector_name: &'static str,
    radius: f32,
    center_angle: f32,
}

fn master_plan_sector(phase_index: u32, era_num: usize) -> RingSectorSpec {
    let specs = [
        RingSectorSpec { ring_name: "Inner Ring (11m)", sector_name: "North Arc", radius: INNER_RING_RADIUS, center_angle: -FRAC_PI_2 },
        RingSectorSpec { ring_name: "Inner Ring (11m)", sector_name: "East Arc", radius: INNER_RING_RADIUS, center_angle: 0.0 },
        RingSectorSpec { ring_name: "Inner Ring (11m)", sector_name: "West Arc", radius: INNER_RING_RADIUS, center_angle: PI },
        RingSectorSpec { ring_name: "Middle Ring (20m)", sector_name: "North Curtain", radius: MIDDLE_RING_RADIUS, center_angle: -FRAC_PI_2 },
        RingSectorSpec { ring_name: "Middle Ring (20m)", sector_name: "South Curtain", radius: MIDDLE_RING_RADIUS, center_angle: FRAC_PI_2 },
        RingSectorSpec { ring_name: "Middle Ring (20m)", sector_name: "East Curtain", radius: MIDDLE_RING_RADIUS, center_angle: 0.0 },
        RingSectorSpec { ring_name: "Middle Ring (20m)", sector_name: "West Curtain", radius: MIDDLE_RING_RADIUS, center_angle: PI },
        RingSectorSpec { ring_name: "Outer Ring (29m)", sector_name: "North Concentric", radius: OUTER_RING_RADIUS, center_angle: -FRAC_PI_2 },
        RingSectorSpec { ring_name: "Outer Ring (29m)", sector_name: "East Concentric", radius: OUTER_RING_RADIUS, center_angle: 0.0 },
        RingSectorSpec { ring_name: "Outer Ring (29m)", sector_name: "South Barbican", radius: OUTER_RING_RADIUS, center_angle: FRAC_PI_2 },
        RingSectorSpec { ring_name: "Outer Ring (29m)", sector_name: "West Concentric", radius: OUTER_RING_RADIUS, center_angle: PI },
    ];
    let idx = ((phase_index.saturating_sub(1) as usize) + era_num.saturating_sub(3) * 2) % specs.len();
    let s = &specs[idx];
    RingSectorSpec {
        ring_name: s.ring_name,
        sector_name: s.sector_name,
        radius: s.radius,
        center_angle: s.center_angle,
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_autonomous_npc_architect(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    state_pack: (Res<SimulationPace>, Res<HistoricalEraState>, Res<VetoedZones>, Option<Res<BlueprintAssets>>),
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: ResMut<BlueprintStats>,
    mut stroke: ResMut<BlueprintStrokeState>,
    mut arch: ResMut<AutonomousArchitectState>,
    mut debate: ResMut<ActiveFeudalDebate>,
    mut feed: ResMut<ActivityFeed>,
    sites_q: Query<&ConstructionSite>,
    built_q: Query<&BuiltStructure>,
    mut lords_q: Query<&mut LordMind>,
    mut overseers_q: Query<&mut SpecialistOverseer>,
) {
    let (pace, era_state, vetoed, bp_assets) = state_pack;
    let Some(assets) = bp_assets else { return };

    if keys.just_pressed(KeyCode::KeyG) {
        if debate.active {
            debate.timer = 0.0;
            return;
        }
        arch.propose_now_requested = true;
    }

    if debate.active {
        return;
    }

    let active_stakes = sites_q.iter().count();
    if pace.is_working() && active_stakes <= 4 {
        arch.countdown = (arch.countdown - time.delta_secs()).max(0.0);
    }

    if !arch.propose_now_requested && (!pace.is_working() || active_stakes > 4 || arch.countdown > 0.0) {
        return;
    }

    arch.propose_now_requested = false;
    arch.countdown = 14.0;

    let existing_pts: Vec<Vec3> = sites_q
        .iter()
        .map(|s| s.position)
        .chain(built_q.iter().map(|b| b.position))
        .collect();

    let mut chosen_pts: Vec<Vec3> = Vec::new();
    let mut chosen_spec = master_plan_sector(arch.phase_index, 1);
    let mut skipped_vetoed = false;
    let trigger_debate_this_cycle = arch.phase_index % 2 == 0;

    for _attempt in 0..11 {
        let spec = master_plan_sector(arch.phase_index, 1);
        arch.phase_index += 1;
        let arc_span = 0.96;
        let steps = 5;
        let mut candidate_pts = Vec::new();
        let mut veto_hits = 0;

        for i in 0..steps {
            let frac = (i as f32 / (steps - 1) as f32) - 0.5;
            let angle = spec.center_angle + frac * arc_span;
            let pt = Vec3::new(angle.cos() * spec.radius, 0.0, angle.sin() * spec.radius);
            if vetoed.is_vetoed(pt, 2.4) {
                veto_hits += 1;
                continue;
            }
            if !existing_pts.iter().any(|p| p.distance(pt) < 1.35) {
                candidate_pts.push(pt);
            }
        }

        if veto_hits > 0 && candidate_pts.len() < 2 {
            skipped_vetoed = true;
            continue;
        }
        if candidate_pts.len() >= 2 {
            chosen_pts = candidate_pts;
            chosen_spec = spec;
            break;
        }
    }

    if chosen_pts.is_empty() {
        return;
    }

    let focus_pt = chosen_pts[chosen_pts.len() / 2];

    // Every 2nd cycle, trigger an organic Feudal Architectural Debate at the target ring sector!
    if trigger_debate_this_cycle {
        debate.active = true;
        debate.ring_name = chosen_spec.ring_name;
        debate.sector_name = chosen_spec.sector_name;
        debate.center_pos = focus_pt;
        debate.sector_pts = chosen_pts;
        debate.timer = 15.0;
        debate.force_winner = None;
        debate.aldric_proposal = "\"We need a wide Gatehouse & Merchant Courtyard here so the turnip carts can pass!\"";
        debate.barnaby_proposal = "\"Madness! We must build a solid Watchtower with double battlements—no doors!\"";

        feed.push(format!(
            "FEUDAL DEBATE STARTED at {} ({})! High Lord Aldric & Sir Barnaby are debating the blueprint (Auto-resolves in 15s, or Click a Lord / press [Z]/[X] to break the tie)!",
            chosen_spec.ring_name, chosen_spec.sector_name
        ));
        return;
    }

    stroke.current_stroke_id = stroke.current_stroke_id.wrapping_add(1).max(10);
    let sid = stroke.current_stroke_id;
    let barnaby_turn = arch.phase_index % 3 == 0;

    execute_sector_staking(
        &mut commands,
        &mut meshes,
        &assets,
        &mut stats,
        &vetoed,
        &era_state,
        &existing_pts,
        &chosen_pts,
        sid,
        barnaby_turn,
    );

    if !barnaby_turn {
        for mut lord in &mut lords_q {
            if lord.archetype == LordArchetype::PracticalFarmerLord {
                lord.target_pos = focus_pt + Vec3::new(0.0, 0.0, -1.4);
                lord.current_thought = if skipped_vetoed {
                    "\"On second thought, that spot had terrible feng shui. I shall leave it open as a courtyard!\"".into()
                } else {
                    "\"A simple fence keeps the sheep in and the wolves out! No need for fancy towers.\"".into()
                };
            }
        }
        feed.push(format!(
            "MASTER PLAN [{} — {}]: High Lord Aldric staked a low fence with a wide Cart Gatehouse (\"No need for fancy towers!\").",
            chosen_spec.ring_name, chosen_spec.sector_name
        ));
    } else {
        for mut lord in &mut lords_q {
            if lord.archetype == LordArchetype::ParanoidMartialVassal {
                lord.target_pos = focus_pt + Vec3::new(0.0, 0.0, -1.4);
                lord.current_thought = "\"Gates are just holes waiting for bandits! Let the peasants climb over with ladders!\"".into();
            }
        }
        feed.push(format!(
            "MASTER PLAN [{} — {}]: Sir Barnaby staked a Watchtower & Wall with NO Gate (\"Gates are just holes waiting for bandits!\").",
            chosen_spec.ring_name, chosen_spec.sector_name
        ));
        feed.push("TIP: Sir Barnaby forgot to leave a gate again! Use [2] Subconscious Whim to drag across his wall and carve an archway (or press [V] -> [Z] to whisper some sense into him).");
    }

    for mut ov in &mut overseers_q {
        if ov.role == OverseerRole::MasterMasonCedric {
            ov.target_pos = focus_pt + Vec3::new(1.1, 0.0, -1.0);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_sector_staking(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &BlueprintAssets,
    stats: &mut BlueprintStats,
    vetoed: &VetoedZones,
    era_state: &HistoricalEraState,
    existing_pts: &[Vec3],
    chosen_pts: &[Vec3],
    sid: u32,
    barnaby_style: bool,
) {
    if chosen_pts.is_empty() {
        return;
    }
    let mut prev_stake = existing_pts
        .iter()
        .copied()
        .filter(|p| {
            let d = p.distance(chosen_pts[0]);
            d > 0.4 && d <= 4.4
        })
        .min_by(|a, b| a.distance(chosen_pts[0]).partial_cmp(&b.distance(chosen_pts[0])).unwrap());

    if !barnaby_style {
        // High Lord Aldric's Proposal: Low Palisade Fence + Central Cart Gatehouse Archway
        let mid_idx = chosen_pts.len() / 2;
        for (idx, &pt) in chosen_pts.iter().enumerate() {
            let kind = if idx == mid_idx {
                SiteKind::GatehouseArchway
            } else {
                SiteKind::PalisadePost
            };
            spawn_stake_site(commands, meshes, assets, pt, prev_stake, sid, kind);
            stats.stakes_planned += 1;
            prev_stake = Some(pt);
        }
    } else {
        // Sir Barnaby's Proposal: Solid Wall + Corner Watchtower Keep (NO Gatehouse!)
        for &pt in chosen_pts {
            spawn_stake_site(commands, meshes, assets, pt, prev_stake, sid, SiteKind::PalisadePost);
            stats.stakes_planned += 1;
            prev_stake = Some(pt);
        }
        let corner_center = *chosen_pts.last().unwrap();
        if !vetoed.is_vetoed(corner_center, 2.5) {
            let tower_radius = 1.9;
            let stories = if era_state.current.uses_stone_masonry() { 3 } else { 2 };
            let mut t_prev = prev_stake;
            for i in 0..5 {
                let a = (i as f32 / 5.0) * TAU;
                let t_pt = corner_center + Vec3::new(a.cos() * tower_radius, 0.0, a.sin() * tower_radius);
                if !existing_pts.iter().any(|p| p.distance(t_pt) < 1.1) {
                    spawn_stake_site(commands, meshes, assets, t_pt, t_prev, sid, SiteKind::TowerKeepWall { stories });
                    stats.stakes_planned += 1;
                    t_prev = Some(t_pt);
                }
            }
            commands.spawn((
                TowerFoundation { center: corner_center, radius: tower_radius, stories, stroke_id: sid, priority_bell: false },
                Transform::from_translation(corner_center),
                Visibility::default(),
            ));
        }
    }
}

