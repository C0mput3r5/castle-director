pub mod briefing;

use crate::atmosphere::AtmosphereState;
use crate::blueprint::{BlueprintStats, ConstructionSite, DirectorMode, GameSpeed};
use crate::camera::CameraMode;
use crate::economy::HomesteadAutomatedGuilds;
use crate::npc::{
    spawn_single_peasant, ActiveFeudalDebate, ActivityFeed, AutonomousArchitectState, LordArchetype,
    LordMind, Peasant, PeasantRoster, ReeveDecree,
};
use crate::raiders::{spawn_slapstick_bandit_wave, BanditAssets, SkirmishState};
use crate::state::{HistoricalEraState, HoverTooltipState, SimulationPace};
use crate::stl_export::StlExportStatus;
use crate::world::{GroundLog, Stockpile, Tree, FARMHOUSE_POS};
use bevy::prelude::*;
use briefing::spawn_center_modals_and_tooltips;

pub const PARCHMENT_TEXT: Color = Color::srgb(0.95, 0.90, 0.80);
pub const PARCHMENT_BORDER: Color = Color::srgb(0.82, 0.72, 0.50);
pub const DARK_WOOD_BG: Color = Color::srgba(0.14, 0.11, 0.08, 0.91);

pub struct UiHudPlugin;
impl Plugin for UiHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HelpCardState>()
            .add_systems(Startup, setup_hud)
            .add_systems(Update, (handle_help_hotkey, handle_hud_buttons, update_hud_texts));
    }
}

#[derive(Resource)]
pub struct HelpCardState { pub visible: bool }
impl Default for HelpCardState { fn default() -> Self { Self { visible: true } } }

#[derive(Component)] pub struct EraBannerText;
#[derive(Component)] pub struct CountersText;
#[derive(Component)] pub struct ModeStatusText;
#[derive(Component)] pub struct EraObjectiveText;
#[derive(Component)] pub struct ArchitectCountdownText;
#[derive(Component)] pub struct ActivityFeedText;
#[derive(Component)] pub struct BriefingModalPanel;
#[derive(Component)] pub struct ContextTooltipPanel;
#[derive(Component)] pub struct ContextTooltipText;
#[derive(Component)] pub struct FeudalDebateCardPanel;
#[derive(Component)] pub struct FeudalDebateCardText;
#[derive(Component)] pub struct StlBannerPanel;
#[derive(Component)] pub struct StlBannerText;
#[derive(Component)] pub struct MindMeldPanel;
#[derive(Component)] pub struct MindMeldBannerText;
#[derive(Component)] pub struct MindMeldMonologueText;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum HudButtonAction {
    InspectMode, DrawPalisadeMode, SpawnPeasant, EnterLordMind, ToggleWorkPace,
    ProposeNextBlueprint, CycleReeveDecree, ToggleCozyAtmosphere, BlowChallengeHorn,
    AdvanceEra, ExportStlDiorama, ToggleHelpCard, ToggleSpeed,
    BackAldricDebate, BackBarnabyDebate,
}

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, padding: UiRect::all(Val::Px(12.0)), ..default() },
        PickingBehavior::IGNORE,
    )).with_children(|root| {
        // --- TOP ROW: Left Chronicle Status + Right Era Objective & Autonomous Planner ---
        root.spawn((
            Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::FlexStart, ..default() },
            PickingBehavior::IGNORE,
        )).with_children(|top| {
            top.spawn((
                Node { max_width: Val::Px(680.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(11.0)), row_gap: Val::Px(4.0), border: UiRect::all(Val::Px(2.0)), ..default() },
                BackgroundColor(DARK_WOOD_BG), BorderColor(PARCHMENT_BORDER), BorderRadius::all(Val::Px(8.0)),
            )).with_children(|panel| {
                panel.spawn((EraBannerText, Text::new("ERA 1: The Humble Homestead | Golden Summer (Sunlit Day)"), TextFont { font_size: 16.5, ..default() }, TextColor(Color::srgb(0.98, 0.86, 0.45))));
                panel.spawn((CountersText, Text::new("Timber: 3 | Turnips: 18 | Stakes: 5 | Built: 0"), TextFont { font_size: 13.2, ..default() }, TextColor(PARCHMENT_TEXT)));
                panel.spawn((ModeStatusText, Text::new("Tool: [1] Observe & Inspect (Default) | COUNCIL BRIEFING ([Space] Sound Work Horn)"), TextFont { font_size: 12.8, ..default() }, TextColor(Color::srgb(0.65, 0.94, 0.68))));
                panel.spawn((
                    Text::new("Controls: [1] Observe & Hover-Inspect (Left-Drag Pan, Click Stake = Priority Bell, Click Debating Lord = Tie-Break) | [2] Subconscious Whim (Left-Drag Gate/Add, Right-Drag Veto)"),
                    TextFont { font_size: 11.0, ..default() }, TextColor(PARCHMENT_TEXT),
                ));
            });

            top.spawn((
                Node { width: Val::Px(520.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(10.0)), row_gap: Val::Px(4.0), border: UiRect::all(Val::Px(2.0)), ..default() },
                BackgroundColor(DARK_WOOD_BG), BorderColor(PARCHMENT_BORDER), BorderRadius::all(Val::Px(8.0)),
            )).with_children(|obj_col| {
                obj_col.spawn((Text::new("CURRENT ERA OBJECTIVE & AUTONOMOUS ARCHITECT"), TextFont { font_size: 13.5, ..default() }, TextColor(Color::srgb(1.0, 0.86, 0.38))));
                obj_col.spawn((EraObjectiveText, Text::new("Era 1 Objective: Let your Lords & Mason enclose the Homestead [0 / 12 Built]"), TextFont { font_size: 12.2, ..default() }, TextColor(PARCHMENT_TEXT)));
                obj_col.spawn((ArchitectCountdownText, Text::new("Next Autonomous Plan: Waiting for Work Horn [Space]..."), TextFont { font_size: 11.8, ..default() }, TextColor(Color::srgb(0.68, 0.92, 0.98))));
                obj_col.spawn((Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(5.0), row_gap: Val::Px(4.0), margin: UiRect::top(Val::Px(3.0)), ..default() },)).with_children(|btns| {
                    spawn_hud_button(btns, "[G] Propose Next Blueprint Now", HudButtonAction::ProposeNextBlueprint);
                    spawn_hud_button(btns, "[R] Reeve Decree", HudButtonAction::CycleReeveDecree);
                    spawn_hud_button(btns, "[T] Time/Season", HudButtonAction::ToggleCozyAtmosphere);
                    spawn_hud_button(btns, "[B] Turnip Bandits!", HudButtonAction::BlowChallengeHorn);
                    spawn_hud_button(btns, "[E] Advance Era", HudButtonAction::AdvanceEra);
                    spawn_hud_button(btns, "[F12] Export .STL", HudButtonAction::ExportStlDiorama);
                    spawn_hud_button(btns, "[H] Briefing Charter", HudButtonAction::ToggleHelpCard);
                    spawn_hud_button(btns, "1x/3x Speed", HudButtonAction::ToggleSpeed);
                });
            });
        });

        // --- CENTER MODALS: Briefing Charter, Hover Tooltip, Feudal Debate Card, .STL Banner & Mind-Meld Overlay ---
        spawn_center_modals_and_tooltips(root);

        // --- BOTTOM ROW: Chronicle Tapestry Feed + Clickable Primary Director Toolbar ---
        root.spawn((
            Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::FlexEnd, column_gap: Val::Px(10.0), ..default() },
            PickingBehavior::IGNORE,
        )).with_children(|bottom| {
            bottom.spawn((
                Node { width: Val::Px(590.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(10.0)), row_gap: Val::Px(3.0), border: UiRect::all(Val::Px(2.0)), ..default() },
                BackgroundColor(DARK_WOOD_BG), BorderColor(PARCHMENT_BORDER), BorderRadius::all(Val::Px(8.0)), PickingBehavior::IGNORE,
            )).with_children(|feed_box| {
                feed_box.spawn((Text::new("CASTLE CHRONICLE TAPESTRY — LORDS, MASON & SERVANT FEED"), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.98, 0.84, 0.44))));
                feed_box.spawn((ActivityFeedText, Text::new("Initializing autonomous homestead chronicle..."), TextFont { font_size: 11.8, ..default() }, TextColor(PARCHMENT_TEXT)));
            });

            bottom.spawn((
                Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, padding: UiRect::all(Val::Px(8.0)), column_gap: Val::Px(6.0), row_gap: Val::Px(5.0), border: UiRect::all(Val::Px(2.5)), ..default() },
                BackgroundColor(Color::srgba(0.16, 0.12, 0.08, 0.95)), BorderColor(Color::srgb(0.95, 0.80, 0.35)), BorderRadius::all(Val::Px(9.0)),
            )).with_children(|bar| {
                spawn_hud_button(bar, "[1] Observe & Inspect (Default)", HudButtonAction::InspectMode);
                spawn_hud_button(bar, "[2] Subconscious Whim (Adjust Plans)", HudButtonAction::DrawPalisadeMode);
                spawn_hud_button(bar, "[3] Nudge Priority / Summon Helper", HudButtonAction::SpawnPeasant);
                spawn_hud_button(bar, "[V] Enter Lord's Mind", HudButtonAction::EnterLordMind);
                spawn_hud_button(bar, "[Space] Sound Work Horn / Pause & Resume", HudButtonAction::ToggleWorkPace);
            });
        });
    });
}

pub fn spawn_hud_button(parent: &mut ChildBuilder, label: &str, action: HudButtonAction) {
    parent.spawn((
        Button, action,
        Node { padding: UiRect::axes(Val::Px(9.0), Val::Px(4.5)), border: UiRect::all(Val::Px(1.5)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgb(0.24, 0.19, 0.14)), BorderColor(PARCHMENT_BORDER), BorderRadius::all(Val::Px(5.0)),
    )).with_children(|btn| {
        btn.spawn((Text::new(label), TextFont { font_size: 11.4, ..default() }, TextColor(PARCHMENT_TEXT)));
    });
}

fn handle_help_hotkey(keys: Res<ButtonInput<KeyCode>>, cam_mode: Res<CameraMode>, mut help: ResMut<HelpCardState>) {
    if keys.just_pressed(KeyCode::KeyH) || (keys.just_pressed(KeyCode::Tab) && *cam_mode == CameraMode::DirectorRTS) {
        help.visible = !help.visible;
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_hud_buttons(
    mut commands: Commands,
    mut interaction_q: Query<(&Interaction, &HudButtonAction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut mode_speed: (ResMut<DirectorMode>, ResMut<GameSpeed>, ResMut<Time<Virtual>>, ResMut<HelpCardState>, ResMut<StlExportStatus>, ResMut<SimulationPace>),
    mut world_states: (ResMut<HistoricalEraState>, ResMut<ReeveDecree>, ResMut<AtmosphereState>, ResMut<SkirmishState>, ResMut<AutonomousArchitectState>, ResMut<CameraMode>, ResMut<ActiveFeudalDebate>),
    bandit_assets: Option<Res<BanditAssets>>,
    lords_q: Query<(Entity, &LordMind)>,
    mut roster: ResMut<PeasantRoster>,
    mut feed: ResMut<ActivityFeed>,
) {
    let (ref mut mode, ref mut speed, ref mut virtual_time, ref mut help, ref mut stl_status, ref mut pace) = mode_speed;
    let (ref mut era_state, ref mut decree, ref mut atmo, ref mut skirmish, ref mut arch, ref mut cam_mode, ref mut debate) = world_states;

    for (interaction, action, mut bg) in &mut interaction_q {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgb(0.48, 0.38, 0.20));
                match action {
                    HudButtonAction::InspectMode => **mode = DirectorMode::Inspect,
                    HudButtonAction::DrawPalisadeMode => **mode = DirectorMode::DrawPalisade,
                    HudButtonAction::SpawnPeasant => {
                        let ox = ((roster.spawned_count % 5) as f32 - 2.0) * 0.9;
                        let name = spawn_single_peasant(&mut commands, &mut roster, FARMHOUSE_POS + Vec3::new(ox, 0.0, 2.8));
                        feed.push(format!("{name} volunteered from the village to help build!"));
                    }
                    HudButtonAction::EnterLordMind => {
                        if matches!(**cam_mode, CameraMode::DirectorRTS) {
                            if let Some((e, l)) = lords_q.iter().find(|(_, l)| l.archetype == LordArchetype::PracticalFarmerLord).or_else(|| lords_q.iter().next()) {
                                **cam_mode = CameraMode::FirstPersonLord(e);
                                feed.push(format!("MIND-MELD ACTIVE: Viewing through the eyes of {}!", l.name));
                            }
                        } else {
                            **cam_mode = CameraMode::DirectorRTS;
                        }
                    }
                    HudButtonAction::ToggleWorkPace => {
                        **pace = match **pace {
                            SimulationPace::PreWorkBriefing => {
                                help.visible = false;
                                feed.push("WORK HORN SOUNDED [Space]! High Lord Aldric, Mason Cedric & the Peasants begin building the castle!");
                                SimulationPace::Running
                            }
                            SimulationPace::Running => SimulationPace::Paused,
                            SimulationPace::Paused => SimulationPace::Running,
                        };
                    }
                    HudButtonAction::ProposeNextBlueprint => {
                        if **pace == SimulationPace::PreWorkBriefing { **pace = SimulationPace::Running; help.visible = false; }
                        if debate.active { debate.timer = 0.0; } else { arch.propose_now_requested = true; }
                    }
                    HudButtonAction::CycleReeveDecree => { **decree = decree.next(); feed.push(format!("REEVE'S PRIORITY DECREE [R] -> {}", decree.label())); }
                    HudButtonAction::ToggleCozyAtmosphere => { atmo.manual_lock = true; atmo.advance_step(); feed.push(format!("COZY ATMOSPHERE [T] -> {} ({})", atmo.season.label(), atmo.time_phase.label())); }
                    HudButtonAction::BlowChallengeHorn => { if let Some(ref assets) = bandit_assets { spawn_slapstick_bandit_wave(&mut commands, assets, skirmish, &mut feed); } }
                    HudButtonAction::AdvanceEra => { era_state.current = era_state.current.next(); feed.push(format!("HISTORICAL ERA EVOLVED -> {}", era_state.current.banner_title())); }
                    HudButtonAction::ExportStlDiorama => { stl_status.trigger_requested = true; }
                    HudButtonAction::ToggleHelpCard => { help.visible = !help.visible; }
                    HudButtonAction::ToggleSpeed => {
                        speed.multiplier = if (speed.multiplier - 1.0).abs() < 0.1 { 3.0 } else { 1.0 };
                        virtual_time.set_relative_speed(speed.multiplier);
                    }
                    HudButtonAction::BackAldricDebate => { debate.force_winner = Some((LordArchetype::PracticalFarmerLord, true)); }
                    HudButtonAction::BackBarnabyDebate => { debate.force_winner = Some((LordArchetype::ParanoidMartialVassal, true)); }
                }
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.35, 0.28, 0.18)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.24, 0.19, 0.14)),
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_hud_texts(
    res_pack: (Res<Stockpile>, Res<HomesteadAutomatedGuilds>, Res<BlueprintStats>, Res<HistoricalEraState>, Res<HelpCardState>, Res<StlExportStatus>),
    state_pack: (Res<ReeveDecree>, Res<AtmosphereState>, Res<SkirmishState>, Res<DirectorMode>, Res<CameraMode>, Res<GameSpeed>),
    pace_arch: (Res<SimulationPace>, Res<AutonomousArchitectState>, Res<ActiveFeudalDebate>, Res<HoverTooltipState>, Res<ActivityFeed>),
    counts_q: (Query<&Tree>, Query<&GroundLog>, Query<&ConstructionSite>),
    npcs_q: (Query<&LordMind>, Query<&Peasant>),
    mut vis_set: ParamSet<(
        Query<&mut Visibility, With<MindMeldPanel>>,
        Query<&mut Node, With<BriefingModalPanel>>,
        Query<&mut Visibility, With<StlBannerPanel>>,
        Query<&mut Visibility, With<ContextTooltipPanel>>,
        Query<&mut Node, With<FeudalDebateCardPanel>>,
    )>,
    mut text_set: ParamSet<(
        Query<&mut Text, With<CountersText>>,
        Query<&mut Text, With<ModeStatusText>>,
        Query<&mut Text, With<ActivityFeedText>>,
        Query<&mut Text, With<MindMeldBannerText>>,
        Query<&mut Text, With<MindMeldMonologueText>>,
        Query<&mut Text, With<EraBannerText>>,
        Query<&mut Text, With<StlBannerText>>,
        Query<&mut Text, (With<EraObjectiveText>, Without<ArchitectCountdownText>)>,
    )>,
    mut extra_text: (
        Query<&mut Text, (With<ArchitectCountdownText>, Without<EraObjectiveText>, Without<FeudalDebateCardText>)>,
        Query<&mut Text, (With<ContextTooltipText>, Without<EraObjectiveText>, Without<ArchitectCountdownText>, Without<FeudalDebateCardText>)>,
        Query<&mut Text, (With<FeudalDebateCardText>, Without<EraObjectiveText>, Without<ArchitectCountdownText>, Without<ContextTooltipText>)>,
    ),
) {
    let (stockpile, guilds, stats, era_state, help, stl_status) = res_pack;
    let (decree, atmo, skirmish, mode, cam_mode, speed) = state_pack;
    let (pace, arch, debate, tooltip, feed) = pace_arch;
    let (trees_q, ground_logs_q, sites_q) = counts_q;
    let (lords_q, peasants_q) = npcs_q;

    let show_briefing = help.visible || *pace == SimulationPace::PreWorkBriefing;
    if let Ok(mut modal_node) = vis_set.p1().get_single_mut() {
        modal_node.display = if show_briefing { Display::Flex } else { Display::None };
    }
    if let Ok(mut deb_node) = vis_set.p4().get_single_mut() {
        deb_node.display = if debate.active && !show_briefing { Display::Flex } else { Display::None };
    }
    if debate.active {
        if let Ok(mut deb_txt) = extra_text.2.get_single_mut() {
            deb_txt.0 = format!(
                "FEUDAL DEBATE IN PROGRESS at {} ({}) — Auto-Resolves in {:.0}s (or Tie-Break Now!)\n\
                 • High Lord Aldric [Z / Click]: {}\n\
                 • Sir Barnaby [X / Click]: {}",
                debate.ring_name, debate.sector_name, debate.timer, debate.aldric_proposal, debate.barnaby_proposal
            );
        }
    }
    if let Ok(mut tip_vis) = vis_set.p3().get_single_mut() {
        *tip_vis = if tooltip.visible && !show_briefing && !debate.active { Visibility::Inherited } else { Visibility::Hidden };
    }
    if tooltip.visible {
        if let Ok(mut tip_txt) = extra_text.1.get_single_mut() {
            tip_txt.0 = format!("INSPECTING: {}\n• Role: {}\n• How to Interact: {}", tooltip.title, tooltip.what_it_does, tooltip.how_to_interact);
        }
    }
    if let Ok(mut stl_vis) = vis_set.p2().get_single_mut() {
        *stl_vis = if stl_status.banner_timer > 0.0 { Visibility::Inherited } else { Visibility::Hidden };
    }
    if stl_status.banner_timer > 0.0 {
        if let Ok(mut stl_txt) = text_set.p6().get_single_mut() { stl_txt.0 = stl_status.last_banner_message.clone(); }
    }
    if let Ok(mut era_txt) = text_set.p5().get_single_mut() {
        era_txt.0 = format!("{}  |  {} ({})", era_state.current.banner_title(), atmo.season.label(), atmo.time_phase.label());
    }
    if let Ok(mut obj_txt) = text_set.p7().get_single_mut() {
        obj_txt.0 = era_state.current.objective_text(stats.posts_completed);
    }
    if let Ok(mut arch_txt) = extra_text.0.get_single_mut() {
        let active_stakes = sites_q.iter().count();
        arch_txt.0 = if debate.active {
            format!("FEUDAL DEBATE ACTIVE at {} ({:.0}s left — Click a Lord or press [Z]/[X]!)", debate.sector_name, debate.timer)
        } else if !pace.is_working() {
            format!("Next Autonomous Plan: {} (Press [Space] to Sound Work Horn or [G] Now)", arch.next_phase_description())
        } else if active_stakes > 4 {
            format!("Servants building {} active stakes -> Next: {} ([G] Propose Now)", active_stakes, arch.next_phase_description())
        } else {
            format!("Mason Cedric & Lords planning '{}' in {:.0}s... ([G] Propose Now)", arch.next_phase_description(), arch.countdown)
        };
    }
    if let Ok(mut text) = text_set.p0().get_single_mut() {
        let peace_tag = if skirmish.active_bandits > 0 { format!("SKIRMISH ({} Bandits)", skirmish.active_bandits) } else { "Peaceful Sandbox".to_string() };
        text.0 = format!(
            "Timber: {} (+{} felled) | Turnips: {} | Stakes: {} | Built: {} | Servants: {} | Trees: {} | {}",
            stockpile.timber_logs, ground_logs_q.iter().count(), guilds.turnips_stored, stats.stakes_planned, stats.posts_completed, peasants_q.iter().count(), trees_q.iter().count(), peace_tag
        );
    }
    if let Ok(mut text) = text_set.p1().get_single_mut() {
        let tool_label = match *cam_mode {
            CameraMode::FirstPersonLord(_) => "FIRST-PERSON MIND-MELD ([F] Pet Hound | [Tab] Switch | [V] Sky)",
            CameraMode::DirectorRTS => match *mode {
                DirectorMode::Inspect => "[1] Observe & Inspect (Default — Hover & Pan)",
                DirectorMode::DrawPalisade => "[2] Subconscious Whim (Adjust / Nudge Plans)",
            },
        };
        text.0 = format!("Active Tool: {}  |  {}  |  Reeve: {} ({:.0}x)", tool_label, pace.status_label(), decree.label(), speed.multiplier);
    }
    if let Ok(mut vis) = vis_set.p0().get_single_mut() {
        match *cam_mode {
            CameraMode::FirstPersonLord(lord_ent) => {
                *vis = Visibility::Inherited;
                if let Ok(lord) = lords_q.get(lord_ent) {
                    if let Ok(mut banner) = text_set.p3().get_single_mut() { banner.0 = format!("MIND-MELD: Inside the mind of {} ([F] Pet Hound | [Tab] Switch | [V] Sky)", lord.name); }
                    if let Ok(mut mono) = text_set.p4().get_single_mut() { mono.0 = format!("Title: {}  |  Ego: {:.0}/100  |  Whisper Influence: {:.0}/100\nLive Inner Monologue: {}", lord.title, lord.ego, lord.whisper_influence, lord.current_thought); }
                }
            }
            CameraMode::DirectorRTS => *vis = Visibility::Hidden,
        }
    }
    if let Ok(mut text) = text_set.p2().get_single_mut() {
        let mut lines = String::new();
        for lord in &lords_q { lines.push_str(&format!("[{}] {}\n", lord.name, lord.current_thought)); }
        for p in peasants_q.iter().take(3) { lines.push_str(&format!(" • {}\n", p.status_text)); }
        lines.push_str("--- Recent Chronicle ---\n");
        for entry in feed.entries.iter().rev().take(3) { lines.push_str(&format!(" > {}\n", entry)); }
        text.0 = lines.trim_end().to_string();
    }
}
