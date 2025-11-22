use bevy::prelude::*;

use bevy::asset::{load_internal_asset, load_internal_binary_asset};
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::render::render_resource::AsBindGroupShaderType;
use bevy::ui_widgets::{Activate, SliderPrecision, SliderStep, observe};
use bevy::window::PresentMode;

use bevy::feathers::{
    FeathersPlugins,
    controls::{ButtonProps, ButtonVariant, SliderProps, button, slider},
    dark_theme::create_dark_theme,
    theme::{ThemeBackgroundColor, ThemedText, UiTheme},
    tokens,
};

use bevy_rapier2d::prelude::*;

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
            width: percent(100),
            height: percent(20),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..Default::default()
        },
        TabGroup::default(),
        ThemeBackgroundColor(tokens::WINDOW_BG),
        children![
            (
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    column_gap: px(15),
                    ..Default::default()
                },
                Text::new(l10n.get("grid_res")),
                (slider(
                    SliderProps {
                        min: 20.,
                        max: 100.,
                        value: params.grid_resolution as f32
                    },
                    (SliderStep(5.), SliderPrecision(0))
                ))
            ),
            (
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((Text::new(l10n.get("start")), ThemedText))
                ),
                observe(|activate: On<Activate>| {
                    info!("kys");
                })
            )
        ],
    )
}
