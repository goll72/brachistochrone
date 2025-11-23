#![feature(iter_map_windows)]
#![feature(more_float_constants)]
#![feature(stmt_expr_attributes)]

use std::collections::HashMap;
use std::f32;

use nalgebra::Vector2;

use bevy::prelude::*;

use bevy::asset::load_internal_binary_asset;
use bevy::ecs::world::CommandQueue;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::platform::time::Instant;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use bevy::ui_widgets::{
    Activate, SliderPrecision, SliderStep, ValueChange, observe, slider_self_update,
};
use bevy::window::{PresentMode, PrimaryWindow};

use bevy::feathers::{
    FeathersPlugins,
    controls::{ButtonProps, SliderProps, button, slider},
    dark_theme::create_dark_theme,
    theme::{ThemeBackgroundColor, ThemedText, UiTheme},
    tokens,
};

use bevy_rapier2d::prelude::*;

use serde::Deserialize;

#[allow(dead_code)]
mod brachistochrone;
#[allow(unused_imports)]
use brachistochrone::Brachistochrone;

#[derive(Resource, Default, Clone)]
struct BrachistochroneParams {
    start: Vector2<f32>,
    end: Vector2<f32>,
    grid_resolution: u8,
    // These values are taken from the initial window size
    viewport_width: f32,
    viewport_height: f32,
    friction: f32,
}

/// The main body under simulation (rolling on the Brachistochrone-like curve)
#[derive(Component)]
struct MainBody;

/// Segments of the Brachistochrone path/curve
#[derive(Component)]
struct BrachistochronePath;

/// Simulation time UI element
/// Keeps track of the time instant when the simulation began
#[derive(Component)]
enum SimulationTime {
    Valid(Instant),
    Invalid,
    Frozen,
}

#[derive(Resource, Deserialize)]
struct Localization(HashMap<String, String>);

impl Localization {
    fn get(&self, s: &str) -> &String {
        match self.0.get(s) {
            Some(x) => x,
            _ => panic!("Translation key not found: {s}"),
        }
    }
}

const PX_PER_M: f32 = 50.;
const MAIN_BODY_RADIUS: f32 = PX_PER_M * f32::consts::FRAC_1_SQRT_PI;

fn main() {
    let mut app = App::new();

    let url = if cfg!(target_family = "wasm") {
        web_sys::window()
            .map(|x| x.document())
            .flatten()
            .map(|x| x.url().ok())
            .flatten()
            .unwrap_or("/".into())
    } else {
        "/".into()
    };

    let l10n: Localization = {
        let json = match (url.get(0..1), url.get(1..3), url.get(3..4)) {
            (Some("/"), Some(lang), Some("/")) => match lang {
                "pt" => include_str!("assets/translations/pt.json"),
                _ => panic!("Invalid lang"),
            },
            _ => include_str!("assets/translations/en.json"),
        };

        serde_json::from_str(json).unwrap()
    };

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Brachistochrone DP Demo".into(),
                name: Some("brachistochrone-dp-bevy".into()),
                present_mode: PresentMode::AutoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }),
        FeathersPlugins,
    ))
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
        PX_PER_M,
    ))
    .insert_resource(BrachistochroneParams {
        start: Vector2::new(0., 10.),
        end: Vector2::new(10., 0.),
        grid_resolution: 50,
        friction: 0.,
        ..Default::default()
    })
    .insert_resource(l10n)
    .insert_resource(UiTheme(create_dark_theme()))
    .add_systems(Startup, setup)
    .add_systems(Update, consume_brachistochrone_path)
    .add_systems(Update, show_simulation_time);

    load_internal_binary_asset!(
        app,
        TextFont::default().font,
        "assets/fonts/FiraCode-Regular.ttf",
        |bytes: &[u8], _: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
    );

    app.run();
}

fn setup(
    mut commands: Commands,
    mut params: ResMut<BrachistochroneParams>,
    l10n: Res<Localization>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    params.viewport_width = window.width();
    params.viewport_height = window.height();

    commands.spawn(Camera2d::default());
    commands.spawn(brachistochrone_ui(params.into(), l10n));
    commands.spawn(simulation_time_ui());
}

fn show_simulation_time(
    params: Res<BrachistochroneParams>,
    l10n: Res<Localization>,
    mut sim_time_query: Query<(&mut Text, &mut SimulationTime)>,
    main_body_pos_query: Query<&Transform, With<MainBody>>,
) {
    let Ok((mut text, mut sim_time)) = sim_time_query.single_mut() else {
        return;
    };

    let elapsed = match *sim_time {
        SimulationTime::Valid(sim_time) => sim_time.elapsed(),
        SimulationTime::Invalid => {
            text.clear();
            return;
        }
        SimulationTime::Frozen => return,
    };

    let millis = elapsed.as_millis();
    let secs = millis / 1000;

    text.0 = format!(
        "{:02}:{:02}.{:02} [{}]",
        secs / 60,
        secs % 60,
        (millis % 1000) / 10,
        l10n.get("inaccuracy_disclaimer")
    );

    let Ok(main_body_pos) = main_body_pos_query.single() else {
        return;
    };

    // Add the ball's radius, since the ball's position corresponds to its center
    let dist = main_body_pos
        .translation
        .truncate()
        .distance_squared(coords(params.end.into(), &params) + Vec2::new(0., MAIN_BODY_RADIUS));

    if dist < PX_PER_M / 2. {
        *sim_time = SimulationTime::Frozen;
    }
}

#[derive(Component)]
enum StartButtonMarker {
    Start,
    Waiting,
    Reset,
}

/// Transforms coordinates from the physics simulation space to the Bevy
/// window-based coordinates, used for rendering and entity positioning.
///
/// For simplicity, assumes that the window size remains fixed after initialization.
fn coords(r: Vec2, params: &BrachistochroneParams) -> Vec2 {
    Vec2::new(
        r.x * PX_PER_M - params.viewport_width / 4.,
        r.y * PX_PER_M - 3. * params.viewport_height / 8.,
    )
}

/// Menu on the top right corner to allow setting simulation parameters as well as starting/stopping the simulation
fn brachistochrone_ui(params: Res<BrachistochroneParams>, l10n: Res<Localization>) -> impl Bundle {
    macro_rules! label {
        ($translation_key:literal) => {
            (
                Node {
                    padding: UiRect::axes(px(5), px(2)),
                    ..Default::default()
                },
                children![(
                    Text::new(l10n.get($translation_key)),
                    TextFont::from_font_size(14.)
                )],
            )
        };

        ($($t:tt)*) => {
            (
                Node {
                    padding: UiRect::axes(px(5), px(2)),
                    ..Default::default()
                },
                children![(
                    Text::new(format!($($t)*)),
                    TextFont::from_font_size(14.)
                )],
            )
        }
    }

    macro_rules! position_slider {
        ($min:literal, $max:literal, $value:expr, $validate:expr) => {
            (
                Node::default(),
                children![(
                    slider(
                        SliderProps {
                            min: $min,
                            max: $max,
                            value: $value
                        },
                        (SliderStep(0.5), SliderPrecision(1))
                    ),
                    observe(
                        |change: On<ValueChange<f32>>,
                         commands: Commands,
                         params: ResMut<BrachistochroneParams>| {
                            if let Some(_) = $validate(&change, params) {
                                slider_self_update(change, commands);
                            }
                        }
                    )
                )],
            )
        };
    }

    macro_rules! spacer {
        () => {
            (Node {
                height: px(20),
                grid_column: GridPlacement::span(2),
                ..Default::default()
            },)
        };
    }

    (
        Node {
            margin: UiRect::all(px(20)),
            padding: UiRect::all(px(10)),
            width: px(400),
            column_gap: px(30),
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::End,
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::min_content(), GridTrack::fr(1.)],
            grid_template_rows: vec![RepeatedGridTrack::auto(11)],
            ..Default::default()
        },
        TabGroup::default(),
        ThemeBackgroundColor(tokens::WINDOW_BG),
        #[rustfmt::skip]
        children![
            label!("grid_res"),
            (
                // [slider]
                Node::default(),
                children![(
                    slider(
                        SliderProps {
                            min: 10.,
                            max: 90.,
                            value: params.grid_resolution as f32
                        },
                        (SliderStep(5.), SliderPrecision(0))
                    ),
                    observe(
                        |change: On<ValueChange<f32>>,
                         commands: Commands,
                         mut params: ResMut<BrachistochroneParams>| {
                            params.grid_resolution = change.value as u8;
                            slider_self_update(change, commands);
                        }
                    )
                )]
            ),
            spacer!(),
            label!("friction"),
            (
                // [slider]
                Node::default(),
                children![(
                    slider(
                        SliderProps {
                            min: 0.,
                            max: 0.99,
                            value: params.friction
                        },
                        (SliderStep(0.05), SliderPrecision(2))
                    ),
                    observe(
                        |change: On<ValueChange<f32>>,
                         commands: Commands,
                         mut params: ResMut<BrachistochroneParams>| {
                            params.friction = change.value;
                            slider_self_update(change, commands);
                        }
                    )
                )]
            ),
            spacer!(),
            label!("{} [x]", l10n.get("initial_pos")),
            position_slider!(
                0.,
                10.,
                params.start.x,
                |change: &On<ValueChange<f32>>, mut params: ResMut<BrachistochroneParams>|
                    (params.end.x > change.value + 2.).then(|| params.start.x = change.value)
            ),
            label!("{} [y]", l10n.get("initial_pos")),
            position_slider!(
                0.,
                10.,
                params.start.y,
                |change: &On<ValueChange<f32>>, mut params: ResMut<BrachistochroneParams>|
                    (params.end.y < change.value - 2.).then(|| params.start.y = change.value)
            ),
            spacer!(),
            label!("{} [x]", l10n.get("final_pos")),
            position_slider!(
                0.,
                10.,
                params.end.x,
                |change: &On<ValueChange<f32>>, mut params: ResMut<BrachistochroneParams>|
                    (params.start.x < change.value - 2.).then(|| params.end.x = change.value)
            ),
            label!("{} [y]", l10n.get("final_pos")),
            position_slider!(
                0.,
                10.,
                params.end.y,
                |change: &On<ValueChange<f32>>, mut params: ResMut<BrachistochroneParams>|
                    (params.start.y > change.value + 2.).then(|| params.end.y = change.value)
            ),
            spacer!(),
            (
                // [button "start"/"reset"]
                Node {
                    grid_column: GridPlacement::span(2),
                    ..Default::default()
                },
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new(l10n.get("start")), ThemedText, StartButtonMarker::Start))
                    ),
                    observe(|_: On<Activate>,
                             l10n: Res<Localization>,
                             params: Res<BrachistochroneParams>,
                             mut commands: Commands,
                             gen_path_task: Option<ResMut<GenerateBrachistochronePath>>,
                             mut marker_query: Query<(&mut Text, &mut StartButtonMarker)>,
                             main_body_query: Query<Entity, With<MainBody>>,
                             path_segments_query: Query<Entity, With<BrachistochronePath>>,
                             mut sim_time_query: Query<&mut SimulationTime>| {
                        let Ok(mut sim_time) = sim_time_query.single_mut() else {
                            return;
                        };

                        if let Ok((mut text, mut marker)) = marker_query.single_mut() {
                            match *marker {
                                StartButtonMarker::Start => {
                                    text.replace_range(.., "...");
                                    *marker = StartButtonMarker::Waiting;

                                    // The simulation time should only be set once the path has actually been
                                    // generated, i.e. in `consume_brachistochrone_path`
                                    generate_brachistochrone_path(params, commands, gen_path_task);
                                }
                                StartButtonMarker::Reset => {
                                    text.replace_range(.., l10n.get("start"));
                                    *marker = StartButtonMarker::Start;

                                    *sim_time = SimulationTime::Invalid;

                                    if let Ok(id) = main_body_query.single() {
                                        commands.entity(id).despawn();
                                    }

                                    for id in path_segments_query {
                                        commands.entity(id).despawn();
                                    }
                                }
                                StartButtonMarker::Waiting => ()
                            }
                        }
                    })
                )]
            )
        ],
    )
}

/// UI element on the bottom left corner to display the elapsed simulation time
fn simulation_time_ui() -> impl Bundle {
    (
        Node {
            margin: UiRect::all(px(20)),
            align_self: AlignSelf::End,
            justify_self: JustifySelf::Start,
            ..Default::default()
        },
        children![(
            Text::new(""),
            TextFont::from_font_size(16.),
            SimulationTime::Invalid
        )],
    )
}

#[derive(Resource)]
struct GenerateBrachistochronePath(Task<CommandQueue>);

fn generate_brachistochrone_path(
    params: Res<BrachistochroneParams>,
    mut commands: Commands,
    task: Option<ResMut<GenerateBrachistochronePath>>,
) {
    if let Some(_) = task {
        return;
    }

    let params = params.clone();
    let pool = AsyncComputeTaskPool::get();

    commands.insert_resource(GenerateBrachistochronePath(pool.spawn(async move {
        let mut command_queue = CommandQueue::default();

        let mu = 10. / params.grid_resolution as f32;

        let mut brac = Brachistochrone::new(
            params.grid_resolution as usize,
            mu,
            (1. / mu) * params.start,
            (1. / mu) * params.end,
        );

        brac.solve();

        brac.path_iter((1. / mu) * params.start)
            .map_windows(|[(_, start), (_, end)]| {
                let start = coords(Vec2::from(mu * start), &params);
                let end = coords(Vec2::from(mu * end), &params);

                (start, end)
            })
            .for_each(|(start, end)| {
                command_queue.push(move |world: &mut World| {
                    let mut meshes = world.resource_mut::<Assets<Mesh>>();
                    let mesh = meshes.add(Segment2d::new(start, end));

                    let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
                    let material = materials.add(Color::srgba(1., 1., 1., 1.));

                    world.spawn((
                        Mesh2d(mesh),
                        MeshMaterial2d(material),
                        RigidBody::Fixed,
                        Collider::segment(start, end),
                        BrachistochronePath,
                    ));
                })
            });

        command_queue
    })));
}

/// Once the Brachistochrone path has been generated, consume it, spawn the ball
/// (main simulation body), and change the Start button state to Reset.
fn consume_brachistochrone_path(
    l10n: Res<Localization>,
    params: Res<BrachistochroneParams>,
    mut commands: Commands,
    task: Option<ResMut<GenerateBrachistochronePath>>,
    mut marker_query: Query<(&mut Text, &mut StartButtonMarker)>,
    mut sim_time_query: Query<&mut SimulationTime>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(mut task) = task else {
        return;
    };

    let Some(mut command_queue) = check_ready(&mut task.0) else {
        return;
    };

    commands.remove_resource::<GenerateBrachistochronePath>();

    // Move the ball up and to the right a bit, otherwise it would spawn in the middle of the Brachistochrone
    // path, leading to it clipping up or down ("falling through") unpredictably or getting stuck
    let start = coords(params.start.into(), &params) + Vec2::new(MAIN_BODY_RADIUS, 0.);

    let mesh = meshes.add(Circle::new(MAIN_BODY_RADIUS));
    let material = materials.add(Color::srgba(0.8, 0.2, 0.15, 1.));

    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(material),
        RigidBody::Dynamic,
        Transform::from_xyz(start.x, start.y, 0.),
        Collider::ball(MAIN_BODY_RADIUS),
        Friction::new(params.friction),
        MainBody,
    ));

    commands.append(&mut command_queue);

    if let Ok((mut text, mut marker)) = marker_query.single_mut() {
        text.replace_range(.., l10n.get("reset"));
        *marker = StartButtonMarker::Reset;
    }

    if let Ok(mut sim_time) = sim_time_query.single_mut() {
        *sim_time = SimulationTime::Valid(Instant::now());
    }
}
