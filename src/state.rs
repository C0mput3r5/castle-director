use crate::blueprint::BlueprintStats;
use crate::camera::ExteriorShell;
use crate::npc::ActivityFeed;
use bevy::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HistoricalEraState>()
            .init_resource::<SimulationPace>()
            .init_resource::<HoverTooltipState>()
            .init_resource::<VetoedZones>()
            .add_systems(
                Update,
                (
                    handle_era_advance_hotkey_and_prestige,
                    sync_era_world_landmarks,
                ),
            );
    }
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SimulationPace {
    #[default]
    PreWorkBriefing,
    Running,
    Paused,
}

impl SimulationPace {
    pub fn is_working(self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn status_label(self) -> &'static str {
        match self {
            Self::PreWorkBriefing => "COUNCIL BRIEFING (Servants Gathered — Press [Space] to Sound Work Horn!)",
            Self::Running => "WORK IN PROGRESS ([Space] Pause Work)",
            Self::Paused => "WORK PAUSED ([Space] Resume Work)",
        }
    }
}

#[derive(Resource, Default)]
pub struct HoverTooltipState {
    pub visible: bool,
    pub title: String,
    pub what_it_does: String,
    pub how_to_interact: String,
}

#[derive(Resource, Default)]
pub struct VetoedZones {
    pub positions: Vec<Vec3>,
}

impl VetoedZones {
    pub fn record_veto(&mut self, pos: Vec3) {
        let flat = Vec3::new(pos.x, 0.0, pos.z);
        if !self.positions.iter().any(|p| p.distance(flat) < 1.2) {
            self.positions.push(flat);
        }
    }

    pub fn is_vetoed(&self, pos: Vec3, radius: f32) -> bool {
        let flat = Vec3::new(pos.x, 0.0, pos.z);
        self.positions.iter().any(|v| v.distance(flat) <= radius)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum HistoricalEra {
    #[default]
    Era1Homestead,
    Era2MotteAndBailey,
    Era3NormanStoneKeep,
    Era4ConcentricCastle,
    Era5GrandChateau,
}

impl HistoricalEra {
    pub fn next(self) -> Self {
        match self {
            Self::Era1Homestead => Self::Era2MotteAndBailey,
            Self::Era2MotteAndBailey => Self::Era3NormanStoneKeep,
            Self::Era3NormanStoneKeep => Self::Era4ConcentricCastle,
            Self::Era4ConcentricCastle => Self::Era5GrandChateau,
            Self::Era5GrandChateau => Self::Era1Homestead,
        }
    }

    pub fn banner_title(self) -> &'static str {
        match self {
            Self::Era1Homestead => "ERA 1: The Humble Homestead (Ditch & Timber Palisade)",
            Self::Era2MotteAndBailey => "ERA 2: Motte-and-Bailey (Earthen Motte Mound & Timber Watchtower)",
            Self::Era3NormanStoneKeep => "ERA 3: Norman Stone Keep (Ashlar Masonry, Scaffolding & Crenellations)",
            Self::Era4ConcentricCastle => "ERA 4: Concentric Castle (Double Curtain Walls & Flanking Bastions)",
            Self::Era5GrandChateau => "ERA 5: Grand Château (High Gothic Spires & Slate Roofs)",
        }
    }

    pub fn target_segments(self) -> u32 {
        match self {
            Self::Era1Homestead => 12,
            Self::Era2MotteAndBailey => 20,
            Self::Era3NormanStoneKeep => 28,
            Self::Era4ConcentricCastle => 36,
            Self::Era5GrandChateau => 45,
        }
    }

    pub fn objective_text(self, built: u32) -> String {
        let goal = self.target_segments();
        match self {
            Self::Era1Homestead => format!(
                "Era 1 Objective: Let your Lords & Mason enclose the Homestead with 12 Palisade Segments & a Gatehouse [{built} / {goal} Built]"
            ),
            Self::Era2MotteAndBailey => format!(
                "Era 2 Objective: Expand the Bailey & Watchtower defenses around the Earthen Motte [{built} / {goal} Built]"
            ),
            Self::Era3NormanStoneKeep => format!(
                "Era 3 Objective: Erect thick Norman Ashlar Stone Walls & Keep Towers [{built} / {goal} Built]"
            ),
            Self::Era4ConcentricCastle => format!(
                "Era 4 Objective: Complete Double Concentric Curtain Walls & Flanking Bastions [{built} / {goal} Built]"
            ),
            Self::Era5GrandChateau => format!(
                "Era 5 Objective: Crown the Grand Château with Slate Spires & Royal Curtain Walls [{built} / {goal} Built]"
            ),
        }
    }

    pub fn uses_stone_masonry(self) -> bool {
        matches!(
            self,
            Self::Era3NormanStoneKeep | Self::Era4ConcentricCastle | Self::Era5GrandChateau
        )
    }
}

#[derive(Resource, Default)]
pub struct HistoricalEraState {
    pub current: HistoricalEra,
    pub highest_auto_unlocked: HistoricalEra,
    pub motte_spawned: bool,
    pub quarry_spawned: bool,
}

#[derive(Component)]
pub struct EraLandmark;

fn handle_era_advance_hotkey_and_prestige(
    keys: Res<ButtonInput<KeyCode>>,
    stats: Res<BlueprintStats>,
    mut era_state: ResMut<HistoricalEraState>,
    mut feed: ResMut<ActivityFeed>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        era_state.current = era_state.current.next();
        feed.push(format!(
            "HISTORICAL EVOLUTION [E]: Advanced to {}!",
            era_state.current.banner_title()
        ));
        return;
    }

    let target_auto = if stats.posts_completed >= HistoricalEra::Era4ConcentricCastle.target_segments() {
        HistoricalEra::Era5GrandChateau
    } else if stats.posts_completed >= HistoricalEra::Era3NormanStoneKeep.target_segments() {
        HistoricalEra::Era4ConcentricCastle
    } else if stats.posts_completed >= HistoricalEra::Era2MotteAndBailey.target_segments() {
        HistoricalEra::Era3NormanStoneKeep
    } else if stats.posts_completed >= HistoricalEra::Era1Homestead.target_segments() {
        HistoricalEra::Era2MotteAndBailey
    } else {
        HistoricalEra::Era1Homestead
    };

    if target_auto > era_state.highest_auto_unlocked {
        era_state.highest_auto_unlocked = target_auto;
        era_state.current = target_auto;
        feed.push(format!(
            "ERA OBJECTIVE COMPLETE! The realm celebrates and evolves into {}!",
            era_state.current.banner_title()
        ));
    }
}

fn sync_era_world_landmarks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut era_state: ResMut<HistoricalEraState>,
) {
    if era_state.current >= HistoricalEra::Era2MotteAndBailey && !era_state.motte_spawned {
        era_state.motte_spawned = true;
        let turf_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.36, 0.50, 0.24), perceptual_roughness: 0.95, ..default() });
        let earth_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.44, 0.34, 0.22), perceptual_roughness: 0.96, ..default() });
        commands.spawn((EraLandmark, Transform::from_xyz(-11.0, 0.0, -7.5), Visibility::default())).with_children(|p| {
            p.spawn((ExteriorShell, Mesh3d(meshes.add(Cylinder::new(5.6, 1.4))), MeshMaterial3d(earth_mat), Transform::from_xyz(0.0, 0.7, 0.0)));
            p.spawn((ExteriorShell, Mesh3d(meshes.add(Cylinder::new(4.3, 1.4))), MeshMaterial3d(turf_mat), Transform::from_xyz(0.0, 1.9, 0.0)));
        });
    }

    if era_state.current >= HistoricalEra::Era3NormanStoneKeep && !era_state.quarry_spawned {
        era_state.quarry_spawned = true;
        let stone_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.58, 0.57, 0.54), perceptual_roughness: 0.9, ..default() });
        let kiln_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.72, 0.45, 0.28), perceptual_roughness: 0.88, ..default() });
        commands.spawn((EraLandmark, Transform::from_xyz(11.5, 0.0, -5.5), Visibility::default())).with_children(|p| {
            p.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(4.2, 1.2, 3.2))), MeshMaterial3d(stone_mat.clone()), Transform::from_xyz(0.0, 0.6, 0.0)));
            p.spawn((ExteriorShell, Mesh3d(meshes.add(Cuboid::new(2.6, 1.8, 2.2))), MeshMaterial3d(stone_mat), Transform::from_xyz(-0.6, 1.5, -0.3)));
            p.spawn((ExteriorShell, Mesh3d(meshes.add(Cylinder::new(0.85, 1.7))), MeshMaterial3d(kiln_mat), Transform::from_xyz(1.6, 0.85, 0.8)));
        });
    }
}
