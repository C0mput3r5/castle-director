pub mod architect;
pub mod debates;
pub mod overseers;
pub use architect::*;
pub use debates::*;
pub use overseers::*;

use crate::blueprint::{complete_palisade_site, spawn_delivered_log_on_site, spawn_ditch_and_scaffolding_stage, BlueprintAssets, BlueprintStats, ConstructionSite};
use crate::camera::LordLookDirection;
use crate::economy::{CozyNeeds, STEW_POT_POS};
use crate::state::{HistoricalEraState, SimulationPace};
use crate::world::{spawn_ground_log, spawn_tree_sapling, GroundLog, LogAssets, Stockpile, Tree, FARMHOUSE_POS, STOCKPILE_POS};
use bevy::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::{FRAC_PI_2, TAU};

pub struct NpcAiPlugin;
impl Plugin for NpcAiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivityFeed>().init_resource::<PeasantRoster>().init_resource::<ReeveDecree>()
            .init_resource::<AutonomousArchitectState>().init_resource::<ActiveFeudalDebate>()
            .add_systems(Startup, (setup_npcs, setup_debate_speech_bubbles))
            .add_systems(Update, (handle_spawn_peasant_hotkey, handle_reeve_decree_hotkey, update_autonomous_npc_architect, update_feudal_debate_system, update_lords_ai, update_specialist_overseers_ai, update_peasant_ai));
    }
}

#[derive(Resource)]
pub struct ActivityFeed { pub entries: VecDeque<String> }
impl Default for ActivityFeed { fn default() -> Self { Self { entries: VecDeque::from(["High Lord Aldric & Master Mason Cedric staked the first southern palisade line!".into(), "Council Briefing: Review the Royal Charter & press [Space] to sound the Work Horn!".into()]) } } }
impl ActivityFeed { pub fn push(&mut self, msg: impl Into<String>) { if self.entries.len() >= 8 { self.entries.pop_front(); } self.entries.push_back(msg.into()); } }

#[derive(Resource, Default)]
pub struct PeasantRoster { pub spawned_count: usize, pub npc_mesh: Handle<Mesh>, pub head_mesh: Handle<Mesh>, pub crown_mesh: Handle<Mesh>, pub carried_log_mesh: Handle<Mesh>, pub skin_mat: Handle<StandardMaterial>, pub log_mat: Handle<StandardMaterial>, pub tunic_mats: Vec<Handle<StandardMaterial>> }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LordArchetype { PracticalFarmerLord, ParanoidMartialVassal }

#[derive(Component)]
pub struct LordMind { pub name: &'static str, pub title: &'static str, pub archetype: LordArchetype, pub current_thought: String, pub ego: f32, pub whisper_influence: f32, pub opinions: Vec<(&'static str, &'static str)>, pub target_pos: Vec3, pub pause_timer: f32, pub walk_phase: f32, pub urgency_timer: f32, pub praise_timer: f32, pub rivalry_timer: f32 }
impl LordMind {
    pub fn opinion_of(&self, target_name: &str) -> &'static str { self.opinions.iter().find(|(n, _)| *n == target_name).map(|(_, op)| *op).unwrap_or("Useful pair of hands [+5]") }
    pub fn aura_radius(&self) -> f32 { if self.rivalry_timer > 0.0 { 9.0 } else { 4.6 } }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogSource { Ground(Entity), Stockpile }
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PeasantState { Idle, RestingAtStewPot, ChoppingTree(Entity), FetchingLog { source: LogSource, for_site: Option<Entity> }, HaulingToStockpile, HaulingToBlueprint(Entity), BuildingPalisade(Entity) }
#[derive(Component)]
pub struct Peasant { pub name: &'static str, pub state: PeasantState, pub state_timer: f32, pub anim_phase: f32, pub hop_timer: f32, pub carried_log_entity: Entity, pub status_text: String }

const PEASANT_NAMES: &[&str] = &["Giles", "Hob", "Oswin", "Bram", "Wat", "Cedric Jr.", "Milo", "Edric"];

fn setup_npcs(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut roster: ResMut<PeasantRoster>) {
    roster.npc_mesh = meshes.add(Capsule3d::new(0.24, 0.55)); roster.head_mesh = meshes.add(Sphere::new(0.20));
    roster.crown_mesh = meshes.add(Cone { radius: 0.22, height: 0.28 }); roster.carried_log_mesh = meshes.add(Cylinder::new(0.14, 1.15));
    roster.skin_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.92, 0.76, 0.63), perceptual_roughness: 0.8, ..default() });
    roster.log_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.42, 0.28, 0.15), perceptual_roughness: 0.88, ..default() });
    roster.tunic_mats = [Color::srgb(0.28, 0.48, 0.68), Color::srgb(0.58, 0.42, 0.25), Color::srgb(0.35, 0.55, 0.32), Color::srgb(0.64, 0.52, 0.28)].into_iter().map(|c| materials.add(StandardMaterial { base_color: c, ..default() })).collect();

    let gold_mat = materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.84, 0.18), emissive: LinearRgba::new(0.35, 0.25, 0.02, 1.0), metallic: 0.8, perceptual_roughness: 0.25, ..default() });
    let cape_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.68, 0.14, 0.20), perceptual_roughness: 0.6, ..default() });
    let iron_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.32, 0.34, 0.38), metallic: 0.85, perceptual_roughness: 0.35, ..default() });
    let crimson_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.85, 0.12, 0.16), emissive: LinearRgba::new(0.22, 0.02, 0.02, 1.0), ..default() });
    let cape_mesh = meshes.add(Cuboid::new(0.42, 0.72, 0.10));

    let aldric_ops = vec![("Sir Barnaby", "Sees wolf ambushes behind every turnip bush! [-15]"), ("Giles", "Smells of wet turnip, hauls logs well [+15]"), ("Hob", "Good lad, though he stares at the stew pot [+20]"), ("Oswin", "Whistles dreadfully, swings a sturdy axe [+10]"), ("Bram", "Talks to his axe, best woodcutter in the shire [+25]")];
    commands.spawn((
        LordMind { name: "High Lord Aldric", title: "Aldric — Petty High Lord of the Muddy Acre", archetype: LordArchetype::PracticalFarmerLord, current_thought: "If we stack enough oak timber before frost, we can roof the granary too.".into(), ego: 72.0, whisper_influence: 100.0, opinions: aldric_ops, target_pos: Vec3::new(0.0, 0.0, 4.2), pause_timer: 1.5, walk_phase: 0.0, urgency_timer: 0.0, praise_timer: 0.0, rivalry_timer: 0.0 },
        LordLookDirection::default(), Transform::from_xyz(-1.5, 0.0, 1.8), Visibility::default(),
    )).with_children(|p| {
        p.spawn((Mesh3d(roster.npc_mesh.clone()), MeshMaterial3d(gold_mat.clone()), Transform::from_xyz(0.0, 0.55, 0.0)));
        p.spawn((Mesh3d(cape_mesh.clone()), MeshMaterial3d(cape_mat), Transform::from_xyz(0.0, 0.55, -0.22)));
        p.spawn((Mesh3d(roster.head_mesh.clone()), MeshMaterial3d(roster.skin_mat.clone()), Transform::from_xyz(0.0, 1.12, 0.0)));
        p.spawn((Mesh3d(roster.crown_mesh.clone()), MeshMaterial3d(gold_mat), Transform::from_xyz(0.0, 1.48, 0.0)));
    });

    let barnaby_ops = vec![("High Lord Aldric", "Covets my eastern palisade and cares more for turnips than pikes! [-20]"), ("Giles", "Slouches like a footpad, hammers too slowly [-10]"), ("Hob", "Soft belly, needs pike drill before supper [-5]"), ("Oswin", "Whistling gives away our position to scouts! [-15]"), ("Bram", "Fierce swing! Worthy of the vanguard [+20]")];
    commands.spawn((
        LordMind { name: "Sir Barnaby", title: "Sir Barnaby — Paranoid Martial Vassal", archetype: LordArchetype::ParanoidMartialVassal, current_thought: "Look at those unbuilt stakes! A single badger could breach our southern flank!".into(), ego: 84.0, whisper_influence: 100.0, opinions: barnaby_ops, target_pos: Vec3::new(2.5, 0.0, 5.2), pause_timer: 2.2, walk_phase: 0.0, urgency_timer: 0.0, praise_timer: 0.0, rivalry_timer: 0.0 },
        LordLookDirection::default(), Transform::from_xyz(3.2, 0.0, 3.6), Visibility::default(),
    )).with_children(|p| {
        p.spawn((Mesh3d(roster.npc_mesh.clone()), MeshMaterial3d(iron_mat.clone()), Transform::from_xyz(0.0, 0.55, 0.0)));
        p.spawn((Mesh3d(cape_mesh), MeshMaterial3d(crimson_mat.clone()), Transform::from_xyz(0.0, 0.55, -0.22)));
        p.spawn((Mesh3d(roster.head_mesh.clone()), MeshMaterial3d(iron_mat), Transform::from_xyz(0.0, 1.12, 0.0)));
        p.spawn((Mesh3d(roster.crown_mesh.clone()), MeshMaterial3d(crimson_mat), Transform::from_xyz(0.0, 1.50, 0.0)));
    });

    spawn_specialist_overseers(&mut commands, &mut meshes, &mut materials, &roster);
    for i in 0..4 { let a = (i as f32 / 4.0) * TAU; spawn_single_peasant(&mut commands, &mut roster, Vec3::new(a.cos() * 2.5, 0.0, 1.5 + a.sin() * 2.0)); }
}

pub fn spawn_single_peasant(commands: &mut Commands, roster: &mut PeasantRoster, pos: Vec3) -> &'static str {
    let idx = roster.spawned_count; roster.spawned_count += 1;
    let name = PEASANT_NAMES[idx % PEASANT_NAMES.len()];
    let cozy = CozyNeeds::new(idx);
    let tunic_mat = roster.tunic_mats[idx % roster.tunic_mats.len()].clone();
    let carried_log = commands.spawn((Mesh3d(roster.carried_log_mesh.clone()), MeshMaterial3d(roster.log_mat.clone()), Transform::from_xyz(0.0, 0.68, 0.38).with_rotation(Quat::from_rotation_z(FRAC_PI_2)), Visibility::Hidden)).id();
    let peasant_id = commands.spawn((
        Peasant { name, state: PeasantState::Idle, state_timer: 0.25 * (idx as f32), anim_phase: 0.0, hop_timer: 0.0, carried_log_entity: carried_log, status_text: format!("{name} arrived ({})", cozy.quirky_trait) },
        cozy, Transform::from_translation(pos), Visibility::default(),
    )).with_children(|p| {
        p.spawn((Mesh3d(roster.npc_mesh.clone()), MeshMaterial3d(tunic_mat), Transform::from_xyz(0.0, 0.52, 0.0)));
        p.spawn((Mesh3d(roster.head_mesh.clone()), MeshMaterial3d(roster.skin_mat.clone()), Transform::from_xyz(0.0, 1.05, 0.0)));
    }).id();
    commands.entity(peasant_id).add_child(carried_log); name
}

fn handle_spawn_peasant_hotkey(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, mut roster: ResMut<PeasantRoster>, mut feed: ResMut<ActivityFeed>) {
    if keys.just_pressed(KeyCode::Digit3) {
        let ox = ((roster.spawned_count % 5) as f32 - 2.0) * 0.9;
        feed.push(format!("{} volunteered from the village to help build!", spawn_single_peasant(&mut commands, &mut roster, FARMHOUSE_POS + Vec3::new(ox, 0.0, 2.8))));
    }
}

fn update_lords_ai(
    time: Res<Time>, pace: Res<SimulationPace>, debate: Res<ActiveFeudalDebate>, mut gizmos: Gizmos, stockpile: Res<Stockpile>, sites: Query<&ConstructionSite>,
    peasants: Query<(&Peasant, &Transform), Without<LordMind>>,
    mut lords_q: Query<(Entity, &mut LordMind, &mut Transform, &mut LordLookDirection)>,
) {
    let dt = time.delta_secs();
    let lord_positions: Vec<(Entity, Vec3, &'static str)> = lords_q.iter().map(|(e, l, t, _)| (e, t.translation, l.name)).collect();
    let site_vec: Vec<Vec3> = sites.iter().map(|s| s.position).collect();

    for (ent, mut lord, mut tf, mut look) in &mut lords_q {
        let pos = Vec3::new(tf.translation.x, 0.04, tf.translation.z);
        let aura_color = if lord.urgency_timer > 0.0 { Color::srgb(1.0, 0.32, 0.18) } else if lord.archetype == LordArchetype::PracticalFarmerLord { Color::srgb(1.0, 0.84, 0.22) } else { Color::srgb(0.88, 0.25, 0.32) };
        gizmos.circle(Isometry3d::new(pos, Quat::from_rotation_x(FRAC_PI_2)), lord.aura_radius(), aura_color);
        if !pace.is_working() { continue; }
        lord.whisper_influence = (lord.whisper_influence + dt * 4.5).min(100.0);
        lord.urgency_timer = (lord.urgency_timer - dt).max(0.0);
        lord.praise_timer = (lord.praise_timer - dt).max(0.0);
        lord.rivalry_timer = (lord.rivalry_timer - dt).max(0.0);
        if lord.rivalry_timer > 0.0 && !debate.active {
            if let Some((_, other_pos, _)) = lord_positions.iter().find(|(e, _, _)| *e != ent) { lord.target_pos = (*other_pos + pos) * 0.5; }
        }
        let speed = if lord.urgency_timer > 0.0 || debate.active { 5.4 } else { 2.8 };
        let to_target = Vec3::new(lord.target_pos.x - pos.x, 0.0, lord.target_pos.z - pos.z);
        if to_target.length() > 0.4 {
            let dir = to_target.normalize();
            let next = Vec3::new(pos.x, 0.0, pos.z) + dir * speed * dt;
            lord.walk_phase += dt * (if lord.urgency_timer > 0.0 { 15.0 } else { 9.0 });
            tf.translation = Vec3::new(next.x, lord.walk_phase.sin().abs() * 0.10, next.z);
            tf.look_to(dir, Vec3::Y); look.forward = dir;
        } else if !debate.active {
            tf.translation.y = 0.0; lord.pause_timer -= dt;
            if lord.pause_timer <= 0.0 {
                lord.pause_timer = 3.8;
                let near_other = lord_positions.iter().find(|(e, p, _)| *e != ent && p.distance(pos) < 4.5);
                let near_peasant = peasants.iter().find(|(_, pt)| pt.translation.distance(pos) < 4.5);
                lord.current_thought = if lord.urgency_timer > 0.0 { "\"Bandits could strike before sundown! Drive those oak posts faster!\"".into() }
                    else if lord.praise_timer > 0.0 { "\"Fine honest sweat today—every worker gets a double ladle of warm turnip stew!\"".into() }
                    else if let Some((_, _, other_name)) = near_other { format!("\"Look at {other_name} strutting about as if he owns my palisade line!\"") }
                    else if let Some((p, _)) = near_peasant { format!("\"Watching {}: {}\"", p.name, lord.opinion_of(p.name)) }
                    else if !site_vec.is_empty() {
                        match lord.archetype {
                            LordArchetype::PracticalFarmerLord => format!("\"{} stakes still waiting for timber ({} logs in stockpile).\"", site_vec.len(), stockpile.timber_logs),
                            LordArchetype::ParanoidMartialVassal => format!("\"DANGER! {} unbuilt stake gaps where raiders could slip through!\"", site_vec.len()),
                        }
                    } else { "\"The palisade stands firm! Our homestead is becoming a proper stronghold.\"".into() };
                lord.target_pos = if !site_vec.is_empty() { site_vec[(time.elapsed_secs() as usize + (lord.ego as usize)) % site_vec.len()] + Vec3::new(0.0, 0.0, -1.3) }
                    else if lord.archetype == LordArchetype::PracticalFarmerLord { STOCKPILE_POS + Vec3::new(-1.5, 0.0, 1.5) }
                    else { let a = time.elapsed_secs() * 0.4; Vec3::new(a.cos() * 8.0, 0.0, a.sin() * 8.0) };
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_peasant_ai(
    mut commands: Commands, time: Res<Time>, mut meshes: ResMut<Assets<Mesh>>,
    res_pack: (Option<Res<LogAssets>>, Option<Res<BlueprintAssets>>, Res<HistoricalEraState>, Res<ReeveDecree>, Res<SimulationPace>),
    mut stockpile: ResMut<Stockpile>, mut bp_stats: ResMut<BlueprintStats>, mut feed: ResMut<ActivityFeed>,
    lords_q: Query<(&LordMind, &Transform), Without<Peasant>>,
    overseers_q: Query<(&SpecialistOverseer, &Transform), Without<Peasant>>,
    mut peasants: Query<(Entity, &mut Peasant, &mut CozyNeeds, &mut Transform)>,
    mut trees: Query<(Entity, &mut Tree, &Transform), Without<Peasant>>,
    mut ground_logs: Query<(Entity, &mut GroundLog, &Transform), Without<Peasant>>,
    mut sites: Query<(Entity, &mut ConstructionSite)>,
    mut vis_q: Query<&mut Visibility>,
) {
    let dt = time.delta_secs();
    let (Some(log_assets), Some(bp_assets), era_state, decree, pace) = res_pack else { return };
    if !pace.is_working() { return; }
    let any_praise = lords_q.iter().any(|(l, _)| l.praise_timer > 0.0);

    for (p_ent, mut p, mut cozy, mut p_tf) in &mut peasants {
        cozy.stew_buff_timer = (cozy.stew_buff_timer - dt).max(0.0);
        cozy.hound_morale_timer = (cozy.hound_morale_timer - dt).max(0.0);
        if !matches!(p.state, PeasantState::RestingAtStewPot) { cozy.energy = (cozy.energy - dt * 1.65).max(0.0); }
        let p_pos = Vec3::new(p_tf.translation.x, 0.0, p_tf.translation.z);
        let mut mult = cozy.speed_multiplier();
        for (lord, l_tf) in &lords_q {
            if p_pos.distance(Vec3::new(l_tf.translation.x, 0.0, l_tf.translation.z)) <= lord.aura_radius() { mult = mult.max(if lord.urgency_timer > 0.0 { 2.0 } else { 1.55 }); }
        }
        for (_, ov_tf) in &overseers_q {
            if p_pos.distance(Vec3::new(ov_tf.translation.x, 0.0, ov_tf.translation.z)) <= 4.5 { mult = mult.max(1.65); }
        }
        if matches!(p.state, PeasantState::HaulingToBlueprint(s) | PeasantState::BuildingPalisade(s) if sites.get(s).map_or(false, |(_, st)| st.priority_bell)) { mult = mult.max(2.5); }
        let chop_mult = if any_praise { mult * 1.5 } else { mult };
        let mood_note = cozy.mood_tag();

        if let Ok(mut vis) = vis_q.get_mut(p.carried_log_entity) {
            *vis = if matches!(p.state, PeasantState::HaulingToStockpile | PeasantState::HaulingToBlueprint(_)) { Visibility::Inherited } else { Visibility::Hidden };
        }
        if p.hop_timer > 0.0 {
            p.hop_timer = (p.hop_timer - dt).max(0.0);
            p_tf.translation.y = ((p.hop_timer / 0.55) * std::f32::consts::PI).sin() * 0.65;
            continue;
        }

        match p.state {
            PeasantState::RestingAtStewPot => {
                let seat = STEW_POT_POS + Vec3::new(((p_ent.index() % 3) as f32 - 1.0) * 1.1, 0.0, 1.1);
                if move_towards(&mut p_tf, &mut p.anim_phase, seat, 0.65, 2.8, dt) {
                    p_tf.translation.y = -0.12; cozy.eating_timer += dt;
                    p.status_text = format!("{} is sitting by the campfire eating warm turnip stew ({})", p.name, cozy.quirky_trait);
                    if cozy.eating_timer >= 4.0 && *decree != ReeveDecree::FestivalAndFeastRest {
                        cozy.energy = 100.0; cozy.stew_buff_timer = 26.0; cozy.eating_timer = 0.0; p_tf.translation.y = 0.0;
                        let msg = format!("{} finished warm turnip stew -> 'Well-Fed & Cheerful (+50% Speed)'!", p.name);
                        p.status_text = msg.clone(); feed.push(msg); p.state = PeasantState::Idle;
                    } else if *decree == ReeveDecree::FestivalAndFeastRest { cozy.energy = 100.0; cozy.stew_buff_timer = 35.0; }
                }
            }
            PeasantState::Idle => {
                p_tf.translation.y = 0.0; p.state_timer -= dt;
                if p.state_timer > 0.0 { continue; }
                if cozy.energy < 25.0 || *decree == ReeveDecree::FestivalAndFeastRest {
                    cozy.eating_timer = 0.0; p.state = PeasantState::RestingAtStewPot;
                    p.status_text = format!("{} is strolling to the Stew Pot & Campfire for warm stew!", p.name); continue;
                }
                let allow_build = *decree != ReeveDecree::AllHandsTimberAndQuarrying;
                let allow_chop = *decree != ReeveDecree::AllHandsCastleConstruction;
                if allow_build {
                    if let Some((s_ent, mut site)) = sites.iter_mut().filter(|(_, s)| s.has_log && s.reserved_builder.is_none()).max_by_key(|(_, s)| s.priority_bell as u8) {
                        site.reserved_builder = Some(p_ent); p.state = PeasantState::BuildingPalisade(s_ent);
                        p.status_text = format!("{} is marching to build ({mood_note})", p.name); continue;
                    }
                    if let Some((s_ent, mut site)) = sites.iter_mut().filter(|(_, s)| !s.has_log && s.reserved_log_delivery.is_none()).max_by_key(|(_, s)| s.priority_bell as u8) {
                        if let Some((l_ent, mut g_log, _)) = ground_logs.iter_mut().find(|(_, l, _)| l.reserved_by.is_none()) {
                            g_log.reserved_by = Some(p_ent); site.reserved_log_delivery = Some(p_ent);
                            p.state = PeasantState::FetchingLog { source: LogSource::Ground(l_ent), for_site: Some(s_ent) };
                            p.status_text = format!("{} is fetching a felled log for the stakes ({mood_note})", p.name); continue;
                        } else if stockpile.timber_logs > 0 {
                            stockpile.timber_logs -= 1; site.reserved_log_delivery = Some(p_ent);
                            p.state = PeasantState::FetchingLog { source: LogSource::Stockpile, for_site: Some(s_ent) };
                            p.status_text = format!("{} is drawing a log from the stockpile ({mood_note})", p.name); continue;
                        }
                    }
                }
                if let Some((l_ent, mut g_log, _)) = ground_logs.iter_mut().find(|(_, l, _)| l.reserved_by.is_none()) {
                    g_log.reserved_by = Some(p_ent); p.state = PeasantState::FetchingLog { source: LogSource::Ground(l_ent), for_site: None };
                    p.status_text = format!("{} is going to collect a felled timber log ({mood_note})", p.name); continue;
                }
                if allow_chop || stockpile.timber_logs == 0 {
                    let best_tree = trees.iter().filter(|(_, t, _)| t.reserved_by.is_none())
                        .min_by(|(_, _, a), (_, _, b)| p_pos.distance(a.translation).partial_cmp(&p_pos.distance(b.translation)).unwrap()).map(|(e, _, _)| e);
                    if let Some(t_ent) = best_tree {
                        if let Ok((_, mut tree, _)) = trees.get_mut(t_ent) {
                            tree.reserved_by = Some(p_ent); p.state = PeasantState::ChoppingTree(t_ent);
                            let msg = format!("{} is heading into the woods to fell an oak ({mood_note})", p.name);
                            p.status_text = msg.clone(); feed.push(msg); continue;
                        }
                    }
                }
                p.state_timer = 0.8;
            }
            PeasantState::ChoppingTree(t_ent) => {
                let Ok((_, mut tree, t_tf)) = trees.get_mut(t_ent) else { p.state = PeasantState::Idle; continue; };
                let target = Vec3::new(t_tf.translation.x, 0.0, t_tf.translation.z);
                if move_towards(&mut p_tf, &mut p.anim_phase, target, 1.1, 3.8 * mult, dt) {
                    p.anim_phase += dt * 10.0 * chop_mult; p_tf.translation.y = p.anim_phase.sin().abs() * 0.14;
                    tree.health -= dt * 1.35 * chop_mult;
                    if tree.health <= 0.0 {
                        spawn_ground_log(&mut commands, &log_assets, target + Vec3::new(0.4, 0.0, 0.2), 0.4);
                        spawn_ground_log(&mut commands, &log_assets, target + Vec3::new(-0.4, 0.0, -0.3), 1.9);
                        spawn_tree_sapling(&mut commands, &log_assets, target);
                        commands.entity(t_ent).despawn_recursive();
                        let msg = format!("{} felled a tall timber tree into 2 oak logs (a sapling shoot sprouts in its place)!", p.name);
                        p.status_text = msg.clone(); feed.push(msg); p.state = PeasantState::Idle;
                    }
                }
            }
            PeasantState::FetchingLog { source, for_site } => {
                let target_pos = match source {
                    LogSource::Stockpile => STOCKPILE_POS,
                    LogSource::Ground(l_ent) => { let Ok((_, _, l_tf)) = ground_logs.get(l_ent) else { p.state = PeasantState::Idle; continue; }; Vec3::new(l_tf.translation.x, 0.0, l_tf.translation.z) }
                };
                if move_towards(&mut p_tf, &mut p.anim_phase, target_pos, 0.9, 4.0 * mult, dt) {
                    if let LogSource::Ground(l_ent) = source { commands.entity(l_ent).despawn_recursive(); }
                    if let Some(s_ent) = for_site { p.state = PeasantState::HaulingToBlueprint(s_ent); p.status_text = format!("{} is hauling an oak log to the stakes ({mood_note})", p.name); }
                    else { p.state = PeasantState::HaulingToStockpile; p.status_text = format!("{} is hauling an oak log to the stockpile ({mood_note})", p.name); }
                }
            }
            PeasantState::HaulingToStockpile => {
                if *decree != ReeveDecree::AllHandsTimberAndQuarrying {
                    if let Some((s_ent, mut site)) = sites.iter_mut().find(|(_, s)| !s.has_log && s.reserved_log_delivery.is_none()) { site.reserved_log_delivery = Some(p_ent); p.state = PeasantState::HaulingToBlueprint(s_ent); continue; }
                }
                if move_towards(&mut p_tf, &mut p.anim_phase, STOCKPILE_POS, 1.2, 3.1 * mult, dt) {
                    stockpile.timber_logs += 1; let msg = format!("{} stacked a fresh timber log in the Stockpile ({mood_note})", p.name);
                    p.status_text = msg.clone(); feed.push(msg); p.state = PeasantState::Idle;
                }
            }
            PeasantState::HaulingToBlueprint(s_ent) => {
                if let Ok((_, mut site)) = sites.get_mut(s_ent) {
                    if move_towards(&mut p_tf, &mut p.anim_phase, site.position, 0.85, 3.2 * mult, dt) {
                        site.has_log = true; site.reserved_log_delivery = None; site.reserved_builder = Some(p_ent);
                        site.delivered_log_vis = Some(spawn_delivered_log_on_site(&mut commands, &bp_assets, site.position));
                        p.state = PeasantState::BuildingPalisade(s_ent); p.status_text = format!("{} is hammering a fortification into place!", p.name);
                    }
                } else if let Some((new_s, mut next_site)) = sites
                    .iter_mut()
                    .filter(|(_, s)| !s.has_log && s.reserved_log_delivery.is_none())
                    .min_by(|(_, a), (_, b)| p_pos.distance(a.position).partial_cmp(&p_pos.distance(b.position)).unwrap())
                {
                    next_site.reserved_log_delivery = Some(p_ent); p.state = PeasantState::HaulingToBlueprint(new_s);
                    p.status_text = format!("{} abides without question, redirecting timber to the nearest stake!", p.name);
                } else {
                    p.state = PeasantState::HaulingToStockpile;
                }
            }
            PeasantState::BuildingPalisade(s_ent) => {
                let Ok((_, mut site)) = sites.get_mut(s_ent) else { p.state = PeasantState::Idle; continue; };
                if move_towards(&mut p_tf, &mut p.anim_phase, site.position, 0.9, 3.8 * mult, dt) {
                    p.anim_phase += dt * 14.0 * mult; p_tf.translation.y = p.anim_phase.sin().abs() * 0.16; site.build_progress += dt * 1.25 * mult;
                    if !site.ditch_dug {
                        p.status_text = format!("{} is digging the Stage 1 defensive ditch & rampart!", p.name);
                        if site.build_progress >= 1.0 { spawn_ditch_and_scaffolding_stage(&mut commands, &mut meshes, &bp_assets, &mut site, era_state.current); p.status_text = format!("{} finished the ditch and is erecting Stage 2 walls!", p.name); }
                    } else if site.build_progress >= site.build_required {
                        complete_palisade_site(&mut commands, &mut meshes, &bp_assets, s_ent, &site, &mut bp_stats, &era_state);
                        let msg = format!("{} completed an ExteriorShell fortification segment! ({mood_note})", p.name);
                        p.status_text = msg.clone(); feed.push(msg); p.state = PeasantState::Idle;
                    }
                }
            }
        }
    }
}

fn move_towards(tf: &mut Transform, phase: &mut f32, target: Vec3, stop: f32, speed: f32, dt: f32) -> bool {
    let cur = Vec3::new(tf.translation.x, 0.0, tf.translation.z);
    let delta = Vec3::new(target.x, 0.0, target.z) - cur;
    let dist = delta.length();
    if dist <= stop { tf.translation.y = 0.0; return true; }
    let next = cur + (delta / dist) * (speed * dt).min(dist); *phase += dt * 11.0;
    tf.translation = Vec3::new(next.x, phase.sin().abs() * 0.11, next.z); tf.look_to(delta / dist, Vec3::Y);
    false
}
