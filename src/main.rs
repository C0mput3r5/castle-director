mod atmosphere;
mod blueprint;
mod camera;
mod economy;
mod mind_meld;
mod npc;
mod pets;
mod raiders;
mod state;
mod stl_export;
mod ui;
mod world;

use bevy::prelude::*;
use bevy::window::WindowResolution;

// Architectural Plugin Aliases matching AGENTS.md
pub use atmosphere::AtmospherePlugin as atmosphere_plugin;
pub use blueprint::DirectorBlueprintPlugin as director_blueprint_plugin;
pub use blueprint::ProceduralConstructionPlugin as procedural_construction_plugin;
pub use camera::CameraPlugin as camera_plugin;
pub use economy::EconomyPlugin as economy_plugin;
pub use mind_meld::MindMeldPlugin as mind_meld_plugin;
pub use npc::NpcAiPlugin as npc_ai_plugin;
pub use pets::PetsPlugin as pets_plugin;
pub use raiders::RaidersPlugin as raiders_plugin;
pub use state::StatePlugin as state_plugin;
pub use stl_export::StlExportPlugin as stl_export_plugin;
pub use ui::UiHudPlugin as ui_hud_plugin;
pub use world::ResourcesPlugin as resources_plugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.56, 0.77, 0.94)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Castle Director — Procedural Historical Castle Builder (Eras 1 to 5)".into(),
                resolution: WindowResolution::new(1400.0, 860.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            state_plugin,
            camera_plugin,
            atmosphere_plugin,
            resources_plugin,
            economy_plugin,
            director_blueprint_plugin,
            procedural_construction_plugin,
            npc_ai_plugin,
            pets_plugin,
            raiders_plugin,
            mind_meld_plugin,
            stl_export_plugin,
            ui_hud_plugin,
        ))
        .run();
}

