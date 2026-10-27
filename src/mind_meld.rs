use crate::blueprint::{ConstructionSite, DirectorMode, SiteKind};
use crate::camera::{CameraMode, DirectorCamera};
use crate::npc::{ActivityFeed, LordArchetype, LordMind, Peasant};
use crate::world::{GroundLog, STOCKPILE_POS};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct MindMeldPlugin;

impl Plugin for MindMeldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_floating_opinion_tags)
            .add_systems(
                Update,
                (
                    handle_mind_meld_toggle_and_tab,
                    handle_whisper_inception_hotkeys,
                    apply_subjective_mental_filter_and_gizmos,
                    update_floating_opinion_tags,
                ),
            );
    }
}

#[derive(Component)]
pub struct FloatingOpinionTag {
    pub slot: usize,
}

#[derive(Component)]
pub struct FloatingOpinionText {
    pub slot: usize,
}

#[derive(Clone, Copy)]
pub enum WhisperKind {
    Urgency,
    Praise,
    Rivalry,
}

fn setup_floating_opinion_tags(mut commands: Commands) {
    for slot in 0..6 {
        commands
            .spawn((
                FloatingOpinionTag { slot },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.07, 0.06, 0.86)),
                BorderColor(Color::srgb(0.85, 0.72, 0.35)),
                BorderRadius::all(Val::Px(6.0)),
                Visibility::Hidden,
                PickingBehavior::IGNORE,
            ))
            .with_children(|parent| {
                parent.spawn((
                    FloatingOpinionText { slot },
                    Text::new(""),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.96, 0.94, 0.86)),
                ));
            });
    }
}

fn handle_mind_meld_toggle_and_tab(
    keys: Res<ButtonInput<KeyCode>>,
    mut cam_mode: ResMut<CameraMode>,
    mut dir_mode: ResMut<DirectorMode>,
    mut feed: ResMut<ActivityFeed>,
    mut cam_q: Query<&mut DirectorCamera>,
    mut lords_q: Query<(Entity, &LordMind, &mut Visibility)>,
) {
    // [V] toggles First-Person Mind-Meld view into High Lord Aldric (or exits back to DirectorRTS)
    if keys.just_pressed(KeyCode::KeyV) {
        match *cam_mode {
            CameraMode::DirectorRTS => {
                let target_lord = lords_q
                    .iter()
                    .find(|(_, l, _)| l.archetype == LordArchetype::PracticalFarmerLord)
                    .or_else(|| lords_q.iter().next())
                    .map(|(e, l, _)| (e, l.name));

                if let Some((lord_ent, name)) = target_lord {
                    *cam_mode = CameraMode::FirstPersonLord(lord_ent);
                    *dir_mode = DirectorMode::Inspect;
                    if let Ok(mut cam) = cam_q.get_single_mut() {
                        cam.fp_yaw_offset = 0.0;
                        cam.fp_pitch_offset = -0.06;
                    }
                    for (e, _, mut vis) in &mut lords_q {
                        *vis = if e == lord_ent { Visibility::Hidden } else { Visibility::Inherited };
                    }
                    feed.push(format!("MIND-MELD: Entered the mind of {name}!"));
                }
            }
            CameraMode::FirstPersonLord(_) => {
                *cam_mode = CameraMode::DirectorRTS;
                for (_, _, mut vis) in &mut lords_q {
                    *vis = Visibility::Inherited;
                }
                feed.push("Returned to Sky Director RTS view.".to_string());
            }
        }
    }

    // [Tab] while in First-Person mode switches between High Lord Aldric and Vassal Lord Sir Barnaby
    if keys.just_pressed(KeyCode::Tab) {
        if let CameraMode::FirstPersonLord(current_ent) = *cam_mode {
            let mut all_lords: Vec<(Entity, &'static str)> =
                lords_q.iter().map(|(e, l, _)| (e, l.name)).collect();
            all_lords.sort_by_key(|(_, name)| *name);
            if let Some((next_ent, next_name)) = all_lords.into_iter().find(|(e, _)| *e != current_ent) {
                *cam_mode = CameraMode::FirstPersonLord(next_ent);
                if let Ok(mut cam) = cam_q.get_single_mut() {
                    cam.fp_yaw_offset = 0.0;
                    cam.fp_pitch_offset = -0.06;
                }
                for (e, _, mut vis) in &mut lords_q {
                    *vis = if e == next_ent { Visibility::Hidden } else { Visibility::Inherited };
                }
                feed.push(format!("MIND-MELD: Switched consciousness to {next_name}!"));
            }
        }
    }
}

fn handle_whisper_inception_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    cam_mode: Res<CameraMode>,
    mut feed: ResMut<ActivityFeed>,
    mut sites: Query<&mut ConstructionSite>,
    mut lords_q: Query<(Entity, &mut LordMind, &Transform)>,
    mut peasants_q: Query<&mut Peasant>,
) {
    let CameraMode::FirstPersonLord(possessed_ent) = *cam_mode else { return };
    if keys.just_pressed(KeyCode::KeyZ) {
        apply_whisper(WhisperKind::Urgency, possessed_ent, &mut sites, &mut lords_q, &mut peasants_q, &mut feed);
    } else if keys.just_pressed(KeyCode::KeyX) {
        apply_whisper(WhisperKind::Praise, possessed_ent, &mut sites, &mut lords_q, &mut peasants_q, &mut feed);
    } else if keys.just_pressed(KeyCode::KeyC) {
        apply_whisper(WhisperKind::Rivalry, possessed_ent, &mut sites, &mut lords_q, &mut peasants_q, &mut feed);
    }
}

pub fn apply_whisper(
    kind: WhisperKind,
    possessed_ent: Entity,
    sites: &mut Query<&mut ConstructionSite>,
    lords_q: &mut Query<(Entity, &mut LordMind, &Transform)>,
    peasants_q: &mut Query<&mut Peasant>,
    feed: &mut ActivityFeed,
) {
    match kind {
        WhisperKind::Urgency => {
            if let Ok((_, mut lord, l_tf)) = lords_q.get_mut(possessed_ent) {
                lord.whisper_influence = (lord.whisper_influence - 25.0).max(0.0);
                lord.urgency_timer = 20.0;
                if let Some(mut nearest_site) = sites
                    .iter_mut()
                    .min_by(|a, b| a.position.distance(l_tf.translation).partial_cmp(&b.position.distance(l_tf.translation)).unwrap())
                {
                    lord.target_pos = nearest_site.position;
                    nearest_site.priority_bell = true;
                    if lord.archetype == LordArchetype::ParanoidMartialVassal {
                        nearest_site.kind = SiteKind::GatehouseArchway;
                        lord.current_thought = "\"Fine, fine! An armored Gatehouse Archway here makes tactical sense—build it at once!\"".into();
                        feed.push("WHISPER [Z] -> Whispered sense into Sir Barnaby! He converted his wall stake into an Armored Gatehouse + 2x Urgency Aura!");
                        return;
                    }
                }
                lord.current_thought = "\"Bandits could attack before sundown! Finish those stakes at once!\"".into();
                feed.push(format!("WHISPER [Z] -> {} sprints to the stakes with a 2x Urgency Aura (20s)!", lord.name));
            }
        }
        WhisperKind::Praise => {
            if let Ok((_, mut lord, _)) = lords_q.get_mut(possessed_ent) {
                lord.whisper_influence = (lord.whisper_influence - 20.0).max(0.0);
                lord.praise_timer = 20.0;
                lord.ego = (lord.ego + 5.0).min(100.0);
                lord.current_thought = "\"These workers deserve an extra ladle of warm turnip stew tonight...\"".into();
                for mut p in peasants_q.iter_mut() {
                    p.status_text = format!("{} is beaming with pride over promised turnip stew!", p.name);
                }
                feed.push(format!("WHISPER [X] -> {} praised the workforce! Morale & tree-chopping speed boosted!", lord.name));
            }
        }
        WhisperKind::Rivalry => {
            for (_, mut lord, _) in lords_q.iter_mut() {
                lord.rivalry_timer = 18.0;
                lord.ego = (lord.ego + 8.0).min(100.0);
                lord.current_thought = "\"Is that other Lord slacking off?! Watch me command this homestead!\"".into();
            }
            feed.push("WHISPER [C] -> Feudal Rivalry! Both Lords march to debate, doubling their Aura Radius!".to_string());
        }
    }
}

fn apply_subjective_mental_filter_and_gizmos(
    time: Res<Time>,
    cam_mode: Res<CameraMode>,
    mut ambient: ResMut<AmbientLight>,
    mut clear_color: ResMut<ClearColor>,
    mut gizmos: Gizmos,
    lords_q: Query<&LordMind>,
    ground_logs_q: Query<&Transform, With<GroundLog>>,
    sites_q: Query<&ConstructionSite>,
) {
    let CameraMode::FirstPersonLord(lord_ent) = *cam_mode else {
        ambient.color = Color::srgb(0.88, 0.93, 1.0);
        ambient.brightness = 420.0;
        clear_color.0 = Color::srgb(0.56, 0.77, 0.94);
        return;
    };
    let Ok(lord) = lords_q.get(lord_ent) else { return };
    let pulse = (time.elapsed_secs() * 5.0).sin() * 0.5 + 0.5;

    match lord.archetype {
        LordArchetype::PracticalFarmerLord => {
            // High Lord Aldric (Farmer Filter): Warm golden ambient tint + green resource gizmo rings
            ambient.color = Color::srgb(1.0, 0.90, 0.62);
            ambient.brightness = 540.0;
            clear_color.0 = Color::srgb(0.76, 0.70, 0.50);

            let green = Color::srgb(0.20, 0.95, 0.38);
            gizmos.circle(Isometry3d::new(STOCKPILE_POS + Vec3::Y * 0.12, Quat::from_rotation_x(FRAC_PI_2)), 2.5 + pulse * 0.25, green);
            for log_tf in &ground_logs_q {
                gizmos.circle(Isometry3d::new(log_tf.translation + Vec3::Y * 0.08, Quat::from_rotation_x(FRAC_PI_2)), 0.75 + pulse * 0.15, green);
            }
        }
        LordArchetype::ParanoidMartialVassal => {
            // Sir Barnaby (Paranoid Martial Filter): Cool tense blue/red tint + pulsing Red Warning Gizmos on stakes
            ambient.color = Color::srgb(0.76, 0.54, 0.86);
            ambient.brightness = 330.0;
            clear_color.0 = Color::srgb(0.28, 0.22, 0.38);

            let red_warn = Color::srgb(1.0, 0.15 + pulse * 0.25, 0.15);
            for site in &sites_q {
                let r = 0.55 + pulse * 0.45;
                gizmos.circle(Isometry3d::new(site.position + Vec3::Y * 0.10, Quat::from_rotation_x(FRAC_PI_2)), r, red_warn);
                gizmos.circle(Isometry3d::new(site.position + Vec3::Y * 0.95, Quat::from_rotation_x(FRAC_PI_2)), r * 0.6, red_warn);
                gizmos.line(site.position, site.position + Vec3::Y * (1.8 + pulse * 0.6), red_warn);
            }
        }
    }
}

fn update_floating_opinion_tags(
    cam_mode: Res<CameraMode>,
    cam_q: Query<(&Camera, &GlobalTransform)>,
    lords_q: Query<(Entity, &LordMind, &Transform)>,
    peasants_q: Query<(&Peasant, &Transform)>,
    mut tag_nodes: Query<(&FloatingOpinionTag, &mut Node, &mut Visibility)>,
    mut tag_texts: Query<(&FloatingOpinionText, &mut Text)>,
) {
    let CameraMode::FirstPersonLord(possessed_ent) = *cam_mode else {
        for (_, _, mut vis) in &mut tag_nodes {
            *vis = Visibility::Hidden;
        }
        return;
    };
    let (Ok((camera, cam_gtf)), Ok((_, possessed_lord, possessed_tf))) =
        (cam_q.get_single(), lords_q.get(possessed_ent))
    else {
        return;
    };

    // Collect nearby NPCs (other Lords + Peasants) sorted by distance
    let mut targets: Vec<(Vec3, String)> = Vec::new();
    for (ent, other_lord, t_tf) in &lords_q {
        if ent != possessed_ent && t_tf.translation.distance(possessed_tf.translation) < 28.0 {
            let label = format!(
                "{} (Feudal Peer)\nOpinion: \"{}\"",
                other_lord.name,
                possessed_lord.opinion_of(other_lord.name)
            );
            targets.push((t_tf.translation + Vec3::Y * 1.9, label));
        }
    }
    for (p, p_tf) in &peasants_q {
        if p_tf.translation.distance(possessed_tf.translation) < 26.0 {
            let label = format!(
                "{} — {}\nOpinion: \"{}\"",
                p.name,
                p.status_text,
                possessed_lord.opinion_of(p.name)
            );
            targets.push((p_tf.translation + Vec3::Y * 1.65, label));
        }
    }

    let mut projected: Vec<(Vec2, String)> = Vec::new();
    let cam_fwd = cam_gtf.forward().as_vec3();
    for (world_pos, text) in targets {
        let to_npc = (world_pos - cam_gtf.translation()).normalize_or_zero();
        if cam_fwd.dot(to_npc) > 0.25 {
            if let Ok(screen_pos) = camera.world_to_viewport(cam_gtf, world_pos) {
                projected.push((screen_pos, text));
            }
        }
    }

    for (tag, mut node, mut vis) in &mut tag_nodes {
        if let Some((screen_pos, label)) = projected.get(tag.slot) {
            *vis = Visibility::Inherited;
            node.left = Val::Px((screen_pos.x - 110.0).clamp(12.0, 1120.0));
            node.top = Val::Px((screen_pos.y - 42.0).clamp(90.0, 740.0));
            for (t_slot, mut t_str) in &mut tag_texts {
                if t_slot.slot == tag.slot {
                    t_str.0 = label.clone();
                }
            }
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

