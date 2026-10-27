use crate::blueprint::{BuiltStructure, SiteKind, TemporaryProp};
use crate::camera::{CameraMode, TerrainPlinth};
use crate::npc::ActivityFeed;
use crate::world::{Tree, FARMHOUSE_POS};
use bevy::prelude::*;

pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AtmosphereState>().add_systems(
            Update,
            (
                handle_toggle_cozy_time_season_hotkey,
                update_cozy_sun_sky_and_season_tints,
                sync_and_flicker_castle_braziers,
            ),
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CozySeason {
    #[default]
    GoldenSummer,
    CrispAutumn,
    CozyFrost,
    BloomingSpring,
}

impl CozySeason {
    pub fn next(self) -> Self {
        match self {
            Self::GoldenSummer => Self::CrispAutumn,
            Self::CrispAutumn => Self::CozyFrost,
            Self::CozyFrost => Self::BloomingSpring,
            Self::BloomingSpring => Self::GoldenSummer,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::GoldenSummer => "Golden Summer",
            Self::CrispAutumn => "Crisp Autumn",
            Self::CozyFrost => "Cozy Frost",
            Self::BloomingSpring => "Blooming Spring",
        }
    }

    pub fn grass_color(self) -> Color {
        match self {
            Self::GoldenSummer => Color::srgb(0.36, 0.62, 0.27),
            Self::CrispAutumn => Color::srgb(0.56, 0.54, 0.24),
            Self::CozyFrost => Color::srgb(0.68, 0.78, 0.74),
            Self::BloomingSpring => Color::srgb(0.32, 0.68, 0.34),
        }
    }

    pub fn foliage_color(self) -> Color {
        match self {
            Self::GoldenSummer => Color::srgb(0.20, 0.50, 0.22),
            Self::CrispAutumn => Color::srgb(0.82, 0.42, 0.16),
            Self::CozyFrost => Color::srgb(0.58, 0.72, 0.68),
            Self::BloomingSpring => Color::srgb(0.34, 0.66, 0.32),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TimeOfDayPhase {
    #[default]
    SunlitDay,
    GoldenHourDusk,
    CozyBrazierNight,
    RosyDawn,
}

impl TimeOfDayPhase {
    pub fn next(self) -> Self {
        match self {
            Self::SunlitDay => Self::GoldenHourDusk,
            Self::GoldenHourDusk => Self::CozyBrazierNight,
            Self::CozyBrazierNight => Self::RosyDawn,
            Self::RosyDawn => Self::SunlitDay,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SunlitDay => "Sunlit Day",
            Self::GoldenHourDusk => "Golden Hour Dusk",
            Self::CozyBrazierNight => "Cozy Brazier Night",
            Self::RosyDawn => "Rosy Dawn",
        }
    }

    pub fn braziers_lit(self) -> bool {
        matches!(self, Self::GoldenHourDusk | Self::CozyBrazierNight)
    }
}

#[derive(Resource)]
pub struct AtmosphereState {
    pub season: CozySeason,
    pub time_phase: TimeOfDayPhase,
    pub cycle_timer: f32,
    pub manual_lock: bool,
}

impl Default for AtmosphereState {
    fn default() -> Self {
        Self {
            season: CozySeason::GoldenSummer,
            time_phase: TimeOfDayPhase::SunlitDay,
            cycle_timer: 0.0,
            manual_lock: false,
        }
    }
}

impl AtmosphereState {
    pub fn advance_step(&mut self) {
        self.time_phase = self.time_phase.next();
        if self.time_phase == TimeOfDayPhase::SunlitDay {
            self.season = self.season.next();
        }
    }
}

#[derive(Component)]
pub struct HasBrazierChild;

#[derive(Component)]
pub struct CastleBrazierLight {
    pub phase_seed: f32,
}

fn handle_toggle_cozy_time_season_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    mut atmo: ResMut<AtmosphereState>,
    mut feed: ResMut<ActivityFeed>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        atmo.manual_lock = true;
        atmo.advance_step();
        let brazier_msg = if atmo.time_phase.braziers_lit() {
            " — Warm Castle Braziers & Lanterns ignited!"
        } else {
            ""
        };
        feed.push(format!(
            "COZY ATMOSPHERE [T]: {} ({}){}",
            atmo.season.label(),
            atmo.time_phase.label(),
            brazier_msg
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn update_cozy_sun_sky_and_season_tints(
    time: Res<Time>,
    cam_mode: Res<CameraMode>,
    mut atmo: ResMut<AtmosphereState>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<AmbientLight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sun_q: Query<(&mut DirectionalLight, &mut Transform)>,
    plinth_q: Query<&MeshMaterial3d<StandardMaterial>, With<TerrainPlinth>>,
    tree_children_q: Query<&Children, With<Tree>>,
    mat_handle_q: Query<&MeshMaterial3d<StandardMaterial>, Without<TerrainPlinth>>,
) {
    let dt = time.delta_secs();
    if !atmo.manual_lock {
        atmo.cycle_timer += dt;
        if atmo.cycle_timer >= 45.0 {
            atmo.cycle_timer = 0.0;
            atmo.advance_step();
        }
    }

    // 1. Update Sun directional light angle, warmth, and illuminance
    if let Ok((mut sun, mut sun_tf)) = sun_q.get_single_mut() {
        let (sun_pos, sun_col, lux) = match atmo.time_phase {
            TimeOfDayPhase::SunlitDay => (
                Vec3::new(28.0, 42.0, 22.0),
                Color::srgb(1.0, 0.96, 0.88),
                13_500.0,
            ),
            TimeOfDayPhase::GoldenHourDusk => (
                Vec3::new(-34.0, 16.0, 24.0),
                Color::srgb(1.0, 0.68, 0.36),
                7_800.0,
            ),
            TimeOfDayPhase::CozyBrazierNight => (
                Vec3::new(-20.0, 32.0, -25.0),
                Color::srgb(0.52, 0.64, 0.95),
                2_600.0,
            ),
            TimeOfDayPhase::RosyDawn => (
                Vec3::new(34.0, 18.0, -18.0),
                Color::srgb(1.0, 0.78, 0.66),
                8_500.0,
            ),
        };
        sun.color = sun_col;
        sun.illuminance = lux;
        *sun_tf = Transform::from_translation(sun_pos).looking_at(Vec3::ZERO, Vec3::Y);
    }

    // 2. Update Sky & AmbientLight when in DirectorRTS mode (Mind-Meld has its own mental tint)
    if *cam_mode == CameraMode::DirectorRTS {
        let (sky_col, amb_col, amb_bright) = match atmo.time_phase {
            TimeOfDayPhase::SunlitDay => (
                Color::srgb(0.56, 0.77, 0.94),
                Color::srgb(0.95, 0.93, 0.88),
                420.0,
            ),
            TimeOfDayPhase::GoldenHourDusk => (
                Color::srgb(0.86, 0.52, 0.38),
                Color::srgb(1.0, 0.76, 0.54),
                340.0,
            ),
            TimeOfDayPhase::CozyBrazierNight => (
                Color::srgb(0.11, 0.15, 0.26),
                Color::srgb(0.45, 0.54, 0.78),
                190.0,
            ),
            TimeOfDayPhase::RosyDawn => (
                Color::srgb(0.78, 0.64, 0.74),
                Color::srgb(0.96, 0.82, 0.78),
                360.0,
            ),
        };
        clear_color.0 = sky_col;
        ambient.color = amb_col;
        ambient.brightness = amb_bright;
    }

    // 3. Subtle 4-Season grass & foliage tinting
    let target_grass = atmo.season.grass_color();
    for mat_h in &plinth_q {
        if let Some(mat) = materials.get_mut(&mat_h.0) {
            mat.base_color = target_grass;
        }
    }
    let target_foliage = atmo.season.foliage_color();
    if let Some(children) = tree_children_q.iter().next() {
        for &child in children.iter().skip(1) {
            if let Ok(mat_h) = mat_handle_q.get(child) {
                if let Some(mat) = materials.get_mut(&mat_h.0) {
                    mat.base_color = target_foliage;
                }
            }
        }
    }
}

fn sync_and_flicker_castle_braziers(
    mut commands: Commands,
    time: Res<Time>,
    atmo: Res<AtmosphereState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    new_structures_q: Query<(Entity, &BuiltStructure), Without<HasBrazierChild>>,
    mut braziers_q: Query<(&CastleBrazierLight, &mut PointLight, &mut Visibility)>,
) {
    // Attach warm braziers to newly built Gatehouses & Towers (plus every 3rd wall segment)
    for (ent, built) in &new_structures_q {
        commands.entity(ent).insert(HasBrazierChild);
        let should_mount = matches!(
            built.kind,
            SiteKind::GatehouseArchway | SiteKind::TowerKeepWall { .. }
        ) || (built.stroke_id + (built.position.x.abs() as u32)) % 3 == 0;
        if should_mount {
            let h = match built.kind {
                SiteKind::GatehouseArchway => 3.35,
                SiteKind::TowerKeepWall { stories } => (stories as f32) * 2.3 + 0.4,
                SiteKind::PalisadePost => 2.85,
            };
            let ember_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.58, 0.14),
                emissive: LinearRgba::new(1.8, 0.75, 0.12, 1.0),
                ..default()
            });
            let _ = FARMHOUSE_POS;
            commands.entity(ent).with_children(|p| {
                p.spawn((
                    TemporaryProp,
                    CastleBrazierLight {
                        phase_seed: built.position.x * 1.7 + built.position.z,
                    },
                    Mesh3d(meshes.add(Sphere::new(0.16))),
                    MeshMaterial3d(ember_mat),
                    PointLight {
                        color: Color::srgb(1.0, 0.66, 0.24),
                        intensity: 0.0,
                        range: 10.5,
                        shadows_enabled: false,
                        ..default()
                    },
                    Transform::from_xyz(0.0, h, 0.0),
                    Visibility::Hidden,
                ));
            });
        }
    }

    let lit = atmo.time_phase.braziers_lit();
    let t = time.elapsed_secs();
    for (brazier, mut light, mut vis) in &mut braziers_q {
        if lit {
            *vis = Visibility::Inherited;
            let flicker = ((t * 7.5 + brazier.phase_seed).sin() * 0.18 + 1.0).clamp(0.75, 1.25);
            light.intensity = 34_000.0 * flicker;
        } else {
            *vis = Visibility::Hidden;
            light.intensity = 0.0;
        }
    }
}

