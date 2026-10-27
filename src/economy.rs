use crate::blueprint::TemporaryProp;
use crate::camera::ExteriorShell;
use crate::world::FARMHOUSE_POS;
use bevy::prelude::*;
use std::f32::consts::TAU;

pub const STEW_POT_POS: Vec3 = Vec3::new(-3.8, 0.0, 0.2);
pub const TURNIP_PATCH_POS: Vec3 = Vec3::new(-4.8, 0.0, -3.8);

pub const QUIRKY_TRAITS: &[&str] = &[
    "Turnip Connoisseur",
    "Afraid of Heights",
    "Whistles Loudly",
    "Self-Taught Architect",
    "Nap Enthusiast",
    "Counts Every Nail",
    "Sings to Timber Logs",
    "Secretly Envies the Sheep",
];

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HomesteadAutomatedGuilds>()
            .add_systems(Startup, setup_cozy_homestead_economy)
            .add_systems(Update, (animate_campfire_and_stew_pot, tick_turnip_harvest_guild));
    }
}

#[derive(Resource)]
pub struct HomesteadAutomatedGuilds {
    pub turnips_stored: u32,
    pub stew_servings_ready: u32,
    pub harvest_timer: f32,
}

impl Default for HomesteadAutomatedGuilds {
    fn default() -> Self {
        Self {
            turnips_stored: 18,
            stew_servings_ready: 12,
            harvest_timer: 0.0,
        }
    }
}

#[derive(Component)]
pub struct CozyNeeds {
    pub energy: f32,
    pub stew_buff_timer: f32,
    pub quirky_trait: &'static str,
    pub eating_timer: f32,
    pub hound_morale_timer: f32,
}

impl CozyNeeds {
    pub fn new(index: usize) -> Self {
        let trait_name = QUIRKY_TRAITS[index % QUIRKY_TRAITS.len()];
        Self {
            energy: 65.0 + ((index * 11) % 30) as f32,
            stew_buff_timer: 0.0,
            quirky_trait: trait_name,
            eating_timer: 0.0,
            hound_morale_timer: 0.0,
        }
    }

    pub fn speed_multiplier(&self) -> f32 {
        let mut m = if self.stew_buff_timer > 0.0 {
            1.50
        } else if self.energy < 25.0 {
            0.80
        } else {
            1.0
        };
        if self.hound_morale_timer > 0.0 {
            m *= 1.18;
        }
        m
    }

    pub fn mood_tag(&self) -> &'static str {
        if self.stew_buff_timer > 0.0 {
            "Well-Fed & Cheerful (+50% Speed)"
        } else if self.hound_morale_timer > 0.0 {
            "Cheered by Castle Hound (+18% Speed)"
        } else if self.energy < 25.0 {
            "Rumbling Belly (Strolling to Stew Pot)"
        } else {
            self.quirky_trait
        }
    }
}

#[derive(Component)]
pub struct StewBubble {
    pub phase: f32,
    pub base_pos: Vec3,
}

#[derive(Component)]
pub struct CampfireFlame {
    pub phase: f32,
}

fn setup_cozy_homestead_economy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let soil_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.23, 0.14),
        perceptual_roughness: 0.96,
        ..default()
    });
    let border_timber_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.31, 0.18),
        perceptual_roughness: 0.88,
        ..default()
    });
    let turnip_bulb_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.76, 0.52, 0.82),
        perceptual_roughness: 0.75,
        ..default()
    });
    let turnip_leaf_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.68, 0.24),
        perceptual_roughness: 0.82,
        ..default()
    });

    // 1. Spawn 2 Cozy Turnip Patch Plots near the Farmhouse
    let bulb_mesh = meshes.add(Sphere::new(0.13));
    let leaf_mesh = meshes.add(Cone { radius: 0.11, height: 0.26 });
    for plot_idx in 0..2 {
        let plot_center = TURNIP_PATCH_POS + Vec3::new(0.0, 0.0, plot_idx as f32 * 2.4);
        commands
            .spawn((
                Transform::from_translation(plot_center),
                Visibility::default(),
            ))
            .with_children(|plot| {
                // Raised soil bed (watertight solid ExteriorShell)
                plot.spawn((
                    ExteriorShell,
                    Mesh3d(meshes.add(Cuboid::new(2.8, 0.16, 1.8))),
                    MeshMaterial3d(soil_mat.clone()),
                    Transform::from_xyz(0.0, 0.08, 0.0),
                ));
                // Timber border curb
                plot.spawn((
                    ExteriorShell,
                    Mesh3d(meshes.add(Cuboid::new(2.92, 0.22, 1.92))),
                    MeshMaterial3d(border_timber_mat.clone()),
                    Transform::from_xyz(0.0, 0.06, 0.0),
                ));
                // Rows of plump purple-top turnips
                for row in -1..=1 {
                    for col in -2..=2 {
                        let tx = col as f32 * 0.48;
                        let tz = row as f32 * 0.46;
                        plot.spawn((
                            TemporaryProp,
                            Mesh3d(bulb_mesh.clone()),
                            MeshMaterial3d(turnip_bulb_mat.clone()),
                            Transform::from_xyz(tx, 0.18, tz),
                        ));
                        plot.spawn((
                            TemporaryProp,
                            Mesh3d(leaf_mesh.clone()),
                            MeshMaterial3d(turnip_leaf_mat.clone()),
                            Transform::from_xyz(tx, 0.34, tz),
                        ));
                    }
                }
            });
    }

    // 2. Spawn the Bubbling Homestead Stew Pot & Campfire near the Farmhouse
    let iron_pot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.19, 0.22),
        metallic: 0.85,
        perceptual_roughness: 0.42,
        ..default()
    });
    let stew_broth_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.56, 0.18),
        emissive: LinearRgba::new(0.35, 0.18, 0.02, 1.0),
        ..default()
    });
    let flame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.48, 0.10),
        emissive: LinearRgba::new(1.2, 0.42, 0.05, 1.0),
        ..default()
    });
    let hearth_stone_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.50, 0.47),
        perceptual_roughness: 0.9,
        ..default()
    });

    let _ = FARMHOUSE_POS;
    commands
        .spawn((
            Transform::from_translation(STEW_POT_POS),
            Visibility::default(),
        ))
        .with_children(|hearth| {
            // Stone hearth ring plinth
            hearth.spawn((
                ExteriorShell,
                Mesh3d(meshes.add(Cylinder::new(0.95, 0.18))),
                MeshMaterial3d(hearth_stone_mat),
                Transform::from_xyz(0.0, 0.09, 0.0),
            ));
            // Sitting logs around the campfire for tired peasants
            let bench_mesh = meshes.add(Cylinder::new(0.14, 1.1));
            for i in 0..3 {
                let a = (i as f32 / 3.0) * TAU + 0.4;
                let bx = a.cos() * 1.45;
                let bz = a.sin() * 1.45;
                hearth.spawn((
                    TemporaryProp,
                    Mesh3d(bench_mesh.clone()),
                    MeshMaterial3d(border_timber_mat.clone()),
                    Transform::from_xyz(bx, 0.15, bz)
                        .with_rotation(Quat::from_rotation_y(-a) * Quat::from_rotation_z(1.57)),
                ));
            }
            // Campfire flame cone
            hearth.spawn((
                TemporaryProp,
                CampfireFlame { phase: 0.0 },
                Mesh3d(meshes.add(Cone { radius: 0.34, height: 0.55 })),
                MeshMaterial3d(flame_mat),
                Transform::from_xyz(0.0, 0.32, 0.0),
            ));
            // Iron Cauldron Pot + Golden Turnip Stew Broth
            hearth.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Sphere::new(0.46))),
                MeshMaterial3d(iron_pot_mat),
                Transform::from_xyz(0.0, 0.58, 0.0).with_scale(Vec3::new(1.1, 0.85, 1.1)),
            ));
            hearth.spawn((
                TemporaryProp,
                Mesh3d(meshes.add(Cylinder::new(0.42, 0.08))),
                MeshMaterial3d(stew_broth_mat.clone()),
                Transform::from_xyz(0.0, 0.82, 0.0),
            ));
            // Bubbling stew spheres
            let bubble_mesh = meshes.add(Sphere::new(0.075));
            for b in 0..4 {
                let off = Vec3::new((b as f32 - 1.5) * 0.11, 0.86, ((b % 2) as f32 - 0.5) * 0.14);
                hearth.spawn((
                    TemporaryProp,
                    StewBubble {
                        phase: b as f32 * 1.5,
                        base_pos: off,
                    },
                    Mesh3d(bubble_mesh.clone()),
                    MeshMaterial3d(stew_broth_mat.clone()),
                    Transform::from_translation(off),
                ));
            }
            // Warm cozy point light
            hearth.spawn((
                PointLight {
                    color: Color::srgb(1.0, 0.68, 0.28),
                    intensity: 45_000.0,
                    range: 11.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_xyz(0.0, 1.2, 0.0),
            ));
        });
}

fn animate_campfire_and_stew_pot(
    time: Res<Time>,
    mut bubbles: Query<(&mut StewBubble, &mut Transform), Without<CampfireFlame>>,
    mut flames: Query<(&mut CampfireFlame, &mut Transform), Without<StewBubble>>,
) {
    let dt = time.delta_secs();
    for (mut b, mut tf) in &mut bubbles {
        b.phase += dt * 4.5;
        let pop = (b.phase.sin() * 0.5 + 0.5) * 0.16;
        tf.translation = b.base_pos + Vec3::Y * pop;
        tf.scale = Vec3::splat(0.75 + (b.phase.cos() * 0.35).abs());
    }
    for (mut f, mut tf) in &mut flames {
        f.phase += dt * 9.0;
        let flicker = 1.0 + f.phase.sin() * 0.18;
        tf.scale = Vec3::new(1.0, flicker, 1.0);
    }
}

fn tick_turnip_harvest_guild(time: Res<Time>, mut guilds: ResMut<HomesteadAutomatedGuilds>) {
    guilds.harvest_timer += time.delta_secs();
    if guilds.harvest_timer >= 8.0 {
        guilds.harvest_timer = 0.0;
        guilds.turnips_stored += 2;
        guilds.stew_servings_ready = (guilds.stew_servings_ready + 1).min(30);
    }
}

