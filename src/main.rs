use bevy::prelude::*;

use bevy::asset::load_internal_binary_asset;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::ui_widgets::{
    Activate, SliderPrecision, SliderStep, ValueChange, observe, slider_self_update,
};
use bevy::window::PresentMode;

use bevy::feathers::{
    FeathersPlugins,
    controls::{ButtonProps, ButtonVariant, SliderProps, button, slider},
    dark_theme::create_dark_theme,
    theme::{ThemeBackgroundColor, ThemedText, UiTheme},
    tokens,
};

use bevy_rapier2d::prelude::*;

use rapier2d::parry::either::IntoEither;
use serde::Deserialize;

use std::collections::HashMap;

mod brachistochrone;
use brachistochrone::Brachistochrone;

#[derive(Resource)]
struct BrachistochroneParams {
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,

    grid_resolution: u8,
}

#[derive(Resource, Deserialize)]
struct Localization(HashMap<String, String>);

impl Localization {
    fn get(&self, s: &str) -> &String {
        self.0.get(s).unwrap()
    }
}

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

    let url = String::from("/pt/a");

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
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
    .add_plugins(RapierDebugRenderPlugin::default())
    .insert_resource(BrachistochroneParams {
        start_x: 0.,
        start_y: 10.,
        end_x: 10.,
        end_y: 0.,

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

fn brachistochrone_ui(params: Res<BrachistochroneParams>, l10n: Res<Localization>) -> impl Bundle {
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
        // "Grid Resolution" [slider]
        // [button "Start"]
        children![
            (
                // "Grid Resolution"
                Node {
                    padding: UiRect::axes(px(5), px(2)),
                    ..Default::default()
                },
                children![(
                    Text::new(l10n.get("grid_res")),
                    TextFont::from_font_size(16.)
                )],
            ),
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
            (
                // [button "start"]
                Node {
                    grid_column: GridPlacement::span(2),
                    ..Default::default()
                },
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new(l10n.get("start")), ThemedText))
                    ),
                    observe(|activate: On<Activate>| {
                        info!("kys");
                    })
                )]
            )
        ],
    )
}
