use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use bevy::window::PresentMode;

mod brachistochrone;
use brachistochrone::Brachistochrone;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Brachistochrone Demo".into(),
                name: Some("brachistochrone-dp-bevy".into()),
                present_mode: PresentMode::AutoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
