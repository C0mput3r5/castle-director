use crate::blueprint::{
    BlueprintAssets, BlueprintStats, BlueprintStrokeState, BuiltStructure, ConstructionSite,
    DirectorMode,
};
use crate::camera::CameraMode;
use crate::npc::{
    execute_sector_staking, ActivityFeed, AutonomousArchitectState, LordArchetype, LordMind,
    OverseerRole, SpecialistOverseer,
};
use crate::state::{HistoricalEraState, SimulationPace, VetoedZones};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

#[derive(Resource)]
pub struct ActiveFeudalDebate {
    pub active: bool,
    pub ring_name: &'static str,
    pub sector_name: &'static str,
    pub center_pos: Vec3,
    pub sector_pts: Vec<Vec3>,
    pub timer: f32,
    pub aldric_proposal: &'static str,
    pub barnaby_proposal: &'static str,
    pub force_winner: Option<(LordArchetype, bool)>,
}

impl Default for ActiveFeudalDebate {
    fn default() -> Self {
        Self {
            active: false,
            ring_name: "Inner Ring (11m)",
            sector_name: "East Arc",
            center_pos: Vec3::ZERO,
            sector_pts: Vec::new(),
            timer: 0.0,
            aldric_proposal: "\"We need a wide Gatehouse & Merchant Courtyard here so the turnip carts can pass!\"",
            barnaby_proposal: "\"Madness! We must build a solid Watchtower with double battlements—no doors!\"",
            force_winner: None,
        }
    }
}

#[derive(Component)]
pub struct DebateSpeechBubble {
    pub archetype: LordArchetype,
}

#[derive(Component)]
pub struct DebateSpeechBubbleText {
    pub archetype: LordArchetype,
}

pub fn setup_debate_speech_bubbles(mut commands: Commands) {
    for (arch, border_col) in [
        (LordArchetype::PracticalFarmerLord, Color::srgb(0.98, 0.84, 0.25)),
        (LordArchetype::ParanoidMartialVassal, Color::srgb(0.92, 0.28, 0.32)),
    ] {
        commands
            .spawn((
                DebateSpeechBubble { archetype: arch },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    max_width: Val::Px(265.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                    border: UiRect::all(Val::Px(1.8)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.10, 0.08, 0.06, 0.90)),
                BorderColor(border_col),
                BorderRadius::all(Val::Px(6.0)),
                Visibility::Hidden,
                PickingBehavior::IGNORE,
            ))
            .with_children(|p| {
                p.spawn((
                    DebateSpeechBubbleText { archetype: arch },
                    Text::new(""),
                    TextFont { font_size: 11.5, ..default() },
                    TextColor(Color::srgb(0.96, 0.92, 0.82)),
                ));
            });
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_feudal_debate_system(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    state_pack: (
        Res<SimulationPace>,
        Res<HistoricalEraState>,
        Res<VetoedZones>,
        Res<DirectorMode>,
        Res<CameraMode>,
        Option<Res<BlueprintAssets>>,
    ),
    mut mut_pack: (
        ResMut<ActiveFeudalDebate>,
        ResMut<AutonomousArchitectState>,
        ResMut<Assets<Mesh>>,
        ResMut<BlueprintStats>,
        ResMut<BlueprintStrokeState>,
        ResMut<ActivityFeed>,
    ),
    mut gizmos: Gizmos,
    sites_q: Query<&ConstructionSite>,
    built_q: Query<&BuiltStructure>,
    mut lords_q: Query<(&mut LordMind, &mut Transform), Without<SpecialistOverseer>>,
    mut overseers_q: Query<(&mut SpecialistOverseer, &Transform), Without<LordMind>>,
    mut bubbles_q: (
        Query<(&DebateSpeechBubble, &mut Node, &mut Visibility)>,
        Query<(&DebateSpeechBubbleText, &mut Text)>,
        Query<(&Camera, &GlobalTransform)>,
    ),
) {
    let (pace, era_state, vetoed, dir_mode, cam_mode, bp_assets) = state_pack;
    let (ref mut debate, ref mut arch, ref mut meshes, ref mut stats, ref mut stroke, ref mut feed) = mut_pack;

    if !debate.active {
        for (_, _, mut vis) in &mut bubbles_q.0 {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let center = debate.center_pos;
    let aldric_spot = center + Vec3::new(-1.25, 0.0, 0.0);
    let barnaby_spot = center + Vec3::new(1.25, 0.0, 0.0);
    let cedric_spot = center + Vec3::new(0.0, 0.0, -1.15);

    // 1. Draw subtle Golden & Crimson Feudal Debate Ring on the ground between the Lords
    let pulse = (t * 4.5).sin() * 0.5 + 0.5;
    let gold = Color::srgb(1.0, 0.85, 0.22);
    let crimson = Color::srgb(0.92, 0.24, 0.28);
    gizmos.circle(Isometry3d::new(center + Vec3::Y * 0.06, Quat::from_rotation_x(FRAC_PI_2)), 2.1 + pulse * 0.18, gold);
    gizmos.circle(Isometry3d::new(center + Vec3::Y * 0.09, Quat::from_rotation_x(FRAC_PI_2)), 1.65 - pulse * 0.12, crimson);
    gizmos.line(aldric_spot + Vec3::Y * 0.15, barnaby_spot + Vec3::Y * 0.15, gold);

    // 2. Position & animate High Lord Aldric, Sir Barnaby (~2.5m apart facing each other), and Master Mason Cedric
    let mut aldric_world_head = aldric_spot + Vec3::Y * 2.0;
    let mut barnaby_world_head = barnaby_spot + Vec3::Y * 2.0;
    let mut aldric_ego = 75.0_f32;
    let mut barnaby_ego = 80.0_f32;

    for (mut lord, mut l_tf) in &mut lords_q {
        match lord.archetype {
            LordArchetype::PracticalFarmerLord => {
                lord.target_pos = aldric_spot;
                lord.current_thought = debate.aldric_proposal.into();
                aldric_ego = lord.ego;
                if l_tf.translation.distance(aldric_spot) <= 1.2 {
                    l_tf.translation.y = ((t * 9.0).sin().abs()) * 0.14;
                    l_tf.look_to(Vec3::X, Vec3::Y);
                }
                aldric_world_head = l_tf.translation + Vec3::Y * 2.1;
            }
            LordArchetype::ParanoidMartialVassal => {
                lord.target_pos = barnaby_spot;
                lord.current_thought = debate.barnaby_proposal.into();
                barnaby_ego = lord.ego;
                if l_tf.translation.distance(barnaby_spot) <= 1.2 {
                    l_tf.translation.y = ((t * 9.0 + 1.5).sin().abs()) * 0.14;
                    l_tf.look_to(Vec3::NEG_X, Vec3::Y);
                }
                barnaby_world_head = l_tf.translation + Vec3::Y * 2.1;
            }
        }
    }
    for (mut ov, _) in &mut overseers_q {
        if ov.role == OverseerRole::MasterMasonCedric {
            ov.target_pos = cedric_spot;
        }
    }

    // 3. Project floating speech bubbles above the two debating Lords
    if let Ok((camera, cam_gtf)) = bubbles_q.2.get_single() {
        for (bubble, mut node, mut vis) in &mut bubbles_q.0 {
            let (world_pt, label) = match bubble.archetype {
                LordArchetype::PracticalFarmerLord => (
                    aldric_world_head,
                    format!("Lord Aldric [Z / Click]:\n{}", debate.aldric_proposal),
                ),
                LordArchetype::ParanoidMartialVassal => (
                    barnaby_world_head,
                    format!("Sir Barnaby [X / Click]:\n{}", debate.barnaby_proposal),
                ),
            };
            if let Ok(screen_pt) = camera.world_to_viewport(cam_gtf, world_pt) {
                *vis = Visibility::Inherited;
                let x_off = if bubble.archetype == LordArchetype::PracticalFarmerLord { -245.0 } else { 15.0 };
                node.left = Val::Px((screen_pt.x + x_off).clamp(12.0, 1080.0));
                node.top = Val::Px((screen_pt.y - 54.0).clamp(95.0, 720.0));
                for (t_comp, mut txt) in &mut bubbles_q.1 {
                    if t_comp.archetype == bubble.archetype {
                        txt.0 = label.clone();
                    }
                }
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }

    // 4. Check for Unseen Whim Tie-Breaker:
    //    - Pressing [Z] (Back Aldric) or [X] (Back Barnaby) in DirectorRTS mode
    //    - Left-Clicking either Lord in [1] Observe & Inspect mode
    if *cam_mode == CameraMode::DirectorRTS {
        if keys.just_pressed(KeyCode::KeyZ) {
            debate.force_winner = Some((LordArchetype::PracticalFarmerLord, true));
        } else if keys.just_pressed(KeyCode::KeyX) {
            debate.force_winner = Some((LordArchetype::ParanoidMartialVassal, true));
        }
    }
    if *dir_mode == DirectorMode::Inspect && mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor_pt) = stroke.cursor_ground_pos {
            for (lord, l_tf) in &lords_q {
                if Vec3::new(l_tf.translation.x, 0.0, l_tf.translation.z).distance(cursor_pt) <= 2.1 {
                    debate.force_winner = Some((lord.archetype, true));
                    break;
                }
            }
        }
    }

    // 5. Count down the 15s Hands-Off Auto-Resolve Timer
    if pace.is_working() {
        debate.timer = (debate.timer - dt).max(0.0);
    }
    if debate.timer <= 0.0 && debate.force_winner.is_none() {
        let coin = ((t * 10.0) as u32) % 2 == 0;
        let winner = if (aldric_ego - barnaby_ego).abs() > 8.0 {
            if aldric_ego >= barnaby_ego {
                LordArchetype::PracticalFarmerLord
            } else {
                LordArchetype::ParanoidMartialVassal
            }
        } else if coin {
            LordArchetype::PracticalFarmerLord
        } else {
            LordArchetype::ParanoidMartialVassal
        };
        debate.force_winner = Some((winner, false));
    }

    // 6. Execute Resolution when Either Auto-Resolved or Tie-Broken by Unseen Whim
    if let Some((winner_arch, player_tiebreaker)) = debate.force_winner.take() {
        let Some(assets) = bp_assets else { return };
        let existing_pts: Vec<Vec3> = sites_q
            .iter()
            .map(|s| s.position)
            .chain(built_q.iter().map(|b| b.position))
            .collect();

        stroke.current_stroke_id = stroke.current_stroke_id.wrapping_add(1).max(10);
        let sid = stroke.current_stroke_id;
        let barnaby_won = winner_arch == LordArchetype::ParanoidMartialVassal;

        execute_sector_staking(
            &mut commands,
            meshes,
            &assets,
            stats,
            &vetoed,
            &era_state,
            &existing_pts,
            &debate.sector_pts,
            sid,
            barnaby_won,
        );

        for (mut lord, _) in &mut lords_q {
            if lord.archetype == winner_arch {
                lord.ego = (lord.ego + 6.0).min(100.0);
                lord.current_thought = if player_tiebreaker {
                    "\"Aha! That brilliant counter-argument just came to me in a flash of genius!\"".into()
                } else {
                    "\"Haha! Even Cedric agrees my architectural vision is superior!\"".into()
                };
            } else {
                lord.current_thought = "\"Hmph... fine, have it your way this time, Cedric—stake it out!\"".into();
            }
        }

        let winner_name = if barnaby_won { "Sir Barnaby" } else { "High Lord Aldric" };
        let loser_name = if barnaby_won { "High Lord Aldric" } else { "Sir Barnaby" };
        if player_tiebreaker {
            feed.push(format!(
                "UNSEEN TIE-BREAKER: {winner_name} boasts: \"Aha! That brilliant counter-argument just came to me in a flash of genius!\" ({loser_name} grumbles: \"Fine, stake it out, Cedric!\")"
            ));
        } else {
            feed.push(format!(
                "DEBATE AUTO-RESOLVED: {winner_name} won the architectural debate at {} ({})! Mason Cedric staked the winning plan!",
                debate.ring_name, debate.sector_name
            ));
        }
        if barnaby_won {
            feed.push("TIP: Sir Barnaby won the debate and built a Watchtower with NO Gate! Use [2] Subconscious Whim to drag across his wall and carve an archway if desired.");
        }

        debate.active = false;
        arch.countdown = 14.0;
        for (_, _, mut vis) in &mut bubbles_q.0 {
            *vis = Visibility::Hidden;
        }
    }
}
