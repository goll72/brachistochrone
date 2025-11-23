#![feature(stmt_expr_attributes)]

use nalgebra::Vector2;

use bevy::prelude::*;

use bevy::asset::load_internal_binary_asset;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::ui_widgets::{
    Activate, SliderPrecision, SliderStep, ValueChange, observe, slider_self_update,
};
use bevy::window::PresentMode;

use bevy::feathers::{
    FeathersPlugins,
    controls::{ButtonProps, SliderProps, button, slider},
    dark_theme::create_dark_theme,
    theme::{ThemeBackgroundColor, ThemedText, UiTheme},
    tokens,
};

use bevy_rapier2d::prelude::*;

use serde::Deserialize;

use std::collections::HashMap;

#[allow(dead_code)]
mod brachistochrone;
#[allow(unused_imports)]
use brachistochrone::Brachistochrone;

#[derive(Resource)]
struct BrachistochroneParams {
    start: Vector2<f32>,
    end: Vector2<f32>,
    grid_resolution: u8,
}

/// The main body under simulation (rolling on the Brachistochrone-like curve)
#[derive(Component)]
struct MainBody;

#[derive(Component)]
struct BrachistochronePath;

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

const PX_PER_M: f32 = 100.;

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
    .add_plugins(RapierDebugRenderPlugin::default())
    .insert_resource(BrachistochroneParams {
        start: Vector2::new(0., 10.),
        end: Vector2::new(10., 0.),
        grid_resolution: 50,
    })
    .insert_resource(l10n)
    .insert_resource(UiTheme(create_dark_theme()))
    .add_systems(Startup, setup);

    load_internal_binary_asset!(
        app,
        TextFont::default().font,
        "assets/fonts/FiraCode-Regular.ttf",
        |bytes: &[u8], _: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
    );

    app.run();
}

fn setup(mut commands: Commands, params: Res<BrachistochroneParams>, l10n: Res<Localization>) {
    commands.spawn(Camera2d::default());
    commands.spawn(brachistochrone_ui(params, l10n));
}

#[derive(Component)]
enum StartButtonMarker {
    Start,
    Reset,
}

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

    (
        Node {
            margin: UiRect::all(px(20)),
            padding: UiRect::all(px(10)),
            width: px(475),
            column_gap: px(30),
            align_self: AlignSelf::End,
            justify_self: JustifySelf::End,
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::min_content(), GridTrack::fr(1.)],
            grid_template_rows: vec![RepeatedGridTrack::auto(2)],
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
                            min: 20.,
                            max: 100.,
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
            label!("{} [x]", l10n.get("initial_pos")),
            position_slider!(
                0.,
                10.,
                params.start.x,
                |change: &On<ValueChange<f32>>, mut params: ResMut<BrachistochroneParams>|
                    (params.end.x > change.value).then(|| params.start.x = change.value)
            ),
            label!("{} [y]", l10n.get("initial_pos")),
            position_slider!(
                0.,
                10.,
                params.start.y,
                |change: &On<ValueChange<f32>>, mut params: ResMut<BrachistochroneParams>|
                    (params.end.y < change.value).then(|| params.start.y = change.value)
            ),
            label!("{} [x]", l10n.get("final_pos")),
            position_slider!(
                0.,
                10.,
                params.end.x,
                |change: &On<ValueChange<f32>>, mut params: ResMut<BrachistochroneParams>|
                    (params.start.x < change.value).then(|| params.end.x = change.value)
            ),
            label!("{} [y]", l10n.get("final_pos")),
            position_slider!(
                0.,
                10.,
                params.end.y,
                |change: &On<ValueChange<f32>>, mut params: ResMut<BrachistochroneParams>|
                    (params.start.y > change.value).then(|| params.end.y = change.value)
            ),
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
                             mut marker_query: Query<(&mut Text, &mut StartButtonMarker)>,
                             main_body_query: Query<Entity, With<MainBody>>| {
                        if let Ok((mut text, mut marker)) = marker_query.single_mut() {
                            match *marker {
                                StartButtonMarker::Start => {
                                    text.replace_range(.., l10n.get("reset"));
                                    *marker = StartButtonMarker::Reset;

                                    commands.spawn(RigidBody::Dynamic)
                                        .insert(Transform::from_xyz(PX_PER_M * params.start.x, PX_PER_M * params.start.y, 0.))
                                        .insert(Collider::ball(10.))
                                        .insert(MainBody);
                                }
                                StartButtonMarker::Reset => {
                                    text.replace_range(.., l10n.get("start"));
                                    *marker = StartButtonMarker::Start;

                                    if let Ok(id) = main_body_query.single() {
                                        commands.entity(id).despawn();
                                    }
                                }
                            }
                        }
                    })
                )]
            )
        ],
    )
}
