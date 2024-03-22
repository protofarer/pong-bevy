#![allow(unused)]

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_vector_shapes::prelude::*;
use fps::{fps_counter_showhide, fps_text_update_system, setup_fps_counter};
use systems::*;

mod fps;
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
const SCORE_COLOR: Color = Color::rgb(0., 1., 0.);
const MESSAGE_COLOR: Color = Color::rgb(0., 1., 0.);

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
        .add_plugins(Shape2dPlugin::default())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_state::<GameState>()
        .init_state::<RoundState>()
        .add_systems(Startup, (setup, setup_fps_counter))
        .add_plugins(TheGamePlugin)
        .run();
}

struct TheGamePlugin;
impl Plugin for TheGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                check_for_collisions,
                play_collision_sound,
                process_score,
                // tick,
            )
                .chain()
                .in_set(PlaySet),
        )
        .add_systems(
            Update,
            (
                fps_text_update_system,
                fps_counter_showhide,
                (run_scored).run_if(in_state(RoundState::Scored)),
                run_countdown.run_if(in_state(RoundState::Countdown)),
                run_end.run_if(in_state(GameState::End)),
                (update_score_ui, bevy::window::close_on_esc, run_match).in_set(MatchSet),
                run_menu.run_if(in_state(GameState::Menu)),
            ),
        )
        .add_systems(OnEnter(GameState::Match), setup_match)
        .add_systems(OnEnter(GameState::End), setup_end)
        .add_systems(OnEnter(RoundState::Scored), setup_scored)
        .add_systems(OnEnter(RoundState::Countdown), setup_countdown)
        .add_systems(OnExit(GameState::End), despawn_screen::<OnEndScreen>)
        .add_systems(OnExit(RoundState::Scored), despawn_screen::<OnScoredScreen>)
        .add_systems(
            OnExit(RoundState::Countdown),
            move |to_despawn: Query<Entity, With<OnCountdownScreen>>, cmd: Commands| {
                println!("IN despawn countdown",);
                despawn_screen::<OnCountdownScreen>(to_despawn, cmd);
            },
        )
        .configure_sets(
            Update,
            (
                MainMenuSet.run_if(in_state(GameState::Menu)),
                PlaySet.run_if(in_state(RoundState::In)),
                MatchSet.run_if(in_state(GameState::Match)),
            ),
        )
        .configure_sets(FixedUpdate, (PlaySet.run_if(in_state(RoundState::In)),))
        .add_event::<CollisionEvent>()
        .add_event::<ScoreEvent>();
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Match,
    End,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum RoundState {
    #[default]
    Out,
    In,
    Scored,
    Countdown,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct MainMenuSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct PlaySet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct MatchSet;

#[derive(Component)]
enum Player {
    A,
    B,
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Event)]
enum CollisionEvent {
    Wall,
    Paddle,
    Goal,
}

#[derive(Event)]
enum ScoreEvent {
    A,
    B,
}

#[derive(Resource)]
struct CollisionSound {
    wall: Handle<AudioSource>,
    paddle: Handle<AudioSource>,
    goal: Handle<AudioSource>,
}

#[derive(Component)]
struct Wall;

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    boundary_type: Wall,
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            boundary_type: Wall,
        }
    }
}

enum WallLocation {
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }
    fn size(&self) -> Vec2 {
        let arena_width = RIGHT_WALL - LEFT_WALL;
        assert!(arena_width > 0.);

        match self {
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

#[derive(Component)]
struct Goal;

#[derive(Bundle)]
struct GoalBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    boundary_type: Goal,
}

enum GoalLocation {
    Left,
    Right,
}

impl GoalLocation {
    fn position(&self) -> Vec2 {
        match self {
            GoalLocation::Left => Vec2::new(LEFT_WALL, -GOAL_THICKNESS / 2.),
            GoalLocation::Right => Vec2::new(RIGHT_WALL, -GOAL_THICKNESS / 2.),
        }
    }
    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        assert!(arena_height > 0.);

        match self {
            GoalLocation::Left | GoalLocation::Right => {
                Vec2::new(GOAL_THICKNESS, arena_height + WALL_THICKNESS)
            }
        }
    }
}

impl GoalBundle {
    fn new(location: GoalLocation) -> GoalBundle {
        GoalBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: GOAL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            boundary_type: Goal,
        }
    }
}

#[derive(Resource)]
struct Scores {
    a: usize,
    b: usize,
}

#[derive(Component)]
struct ScoreboardUi(Player);

#[derive(Resource)]
struct Match {
    round_count: usize,
    rounds_total: usize,
}

#[derive(Component, Clone)]
struct OnScoredScreen;

#[derive(Component, Clone)]
struct OnEndScreen;

#[derive(Component)]
struct OnCountdownScreen;

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

#[derive(Component)]
struct CountdownTimedMessage {
    timer: Timer,
    texts: [String; 4],
    cursor: usize,
}
