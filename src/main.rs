#![allow(unused)]

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_vector_shapes::prelude::*;
use countdown::*;
use fps::{fps_counter_showhide, fps_text_update_system, setup_fps_counter};
use match_::{run_match, update_score_ui};
use menu::OnMenuScreen;
use systems::*;

mod countdown;
mod fps;
mod match_;
mod menu;
mod scored;
mod systems;

const PADDLE_SIZE: Vec3 = Vec3::new(20., 150., 0.0);
const GAP_BETWEEN_PADDLE_AND_GOAL: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.;

const BALL_START_POSITION: Vec3 = Vec3::new(0., 0., 1.);
const BALL_RADIUS: f32 = 10.;
const BALL_START_SPEED: f32 = 800.;

const WALL_THICKNESS: f32 = 10.;
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const GOAL_THICKNESS: f32 = 3.;

struct ScorePosition {
    top: Val,
    left: Val,
}

const SCORE_POSITION_TOP: Val = Val::Px(100.);

const SCORE_A_POSITION: ScorePosition = ScorePosition {
    top: SCORE_POSITION_TOP,
    left: Val::Percent(25.),
};

const SCORE_B_POSITION: ScorePosition = ScorePosition {
    top: SCORE_POSITION_TOP,
    left: Val::Percent(75.),
};

const SCORE_FONT_SIZE: f32 = 40.;

const BACKGROUND_COLOR: Color = Color::rgb(0., 0., 0.);
const WALL_COLOR: Color = Color::rgb(1., 1., 1.);
const BALL_COLOR: Color = Color::rgb(1., 1., 1.);
const PADDLE_COLOR: Color = Color::rgb(1., 1., 1.);
const GOAL_COLOR: Color = Color::rgb(1., 1., 1.);
const TEXT_COLOR: Color = Color::rgb(0., 1., 0.);

const PADDLE_A_START_POSITION: Vec3 = Vec3::new(LEFT_WALL + GAP_BETWEEN_PADDLE_AND_GOAL, 0., 0.);
const PADDLE_B_START_POSITION: Vec3 = Vec3::new(RIGHT_WALL - GAP_BETWEEN_PADDLE_AND_GOAL, 0., 0.);

const ROUNDS_TOTAL: usize = 2;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy-pong".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, (fps_text_update_system, fps_counter_showhide))
        .add_plugins(Shape2dPlugin::default())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_state::<GameState>()
        .add_systems(Startup, (setup, setup_fps_counter))
        .add_plugins((
            menu::menu_plugin,
            countdown::countdown_plugin,
            scored::scored_plugin,
            match_::match_plugin,
        ))
        .run();
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Match,
    End,
}

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);
