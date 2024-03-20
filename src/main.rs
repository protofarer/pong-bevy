#![allow(unused)]

use bevy::{
    // math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    // sprite::MaterialMesh2dBundle,
};
use systems::*;

mod systems;

const PADDLE_SIZE: Vec3 = Vec3::new(20., 150., 0.0);
const GAP_BETWEEN_PADDLE_AND_BACKWALL: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.;

const BALL_START_POSITION: Vec3 = Vec3::new(0., 0., 1.);
const BALL_R: f32 = 15.;
const BALL_START_SPEED: f32 = 800.;

const WALL_THICKNESS: f32 = 10.;
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const GOAL_THICKNESS: f32 = 3.;

const SCORE_A_POSITION: Vec3 = Vec3::new(-150., 200., 0.0);
const SCORE_B_POSITION: Vec3 = Vec3::new(150., 200., 0.0);

const SCORE_FONT_SIZE: f32 = 40.;

const BACKGROUND_COLOR: Color = Color::rgb(0., 0., 0.);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const GOAL_COLOR: Color = Color::rgb(0., 0., 0.8);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const MESSAGE_COLOR: Color = Color::rgb(0., 0., 0.9);

const PADDLE_A_START_VEC: Vec3 = Vec3::new(LEFT_WALL + GAP_BETWEEN_PADDLE_AND_BACKWALL, 0., 0.);
const PADDLE_B_START_VEC: Vec3 = Vec3::new(RIGHT_WALL - GAP_BETWEEN_PADDLE_AND_BACKWALL, 0., 0.);

const ROUNDS_TOTAL: usize = 2;

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
                (draw_scores, bevy::window::close_on_esc).in_set(PlaySet),
                run_scored_view.run_if(in_state(RoundState::Scored)),
                round_countdown.run_if(in_state(RoundState::Countdown)),
                run_end.run_if(in_state(GameState::End)),
                run_menu.run_if(in_state(GameState::Menu)),
            ),
        )
        .add_systems(OnEnter(GameState::Match), setup_match)
        .add_systems(OnEnter(RoundState::In), setup_round)
        .add_systems(OnEnter(RoundState::Scored), setup_scored)
        .add_systems(OnEnter(GameState::End), setup_end)
        .add_systems(OnExit(RoundState::Scored), despawn_screen::<OnScoredScreen>)
        .add_systems(OnExit(GameState::End), despawn_screen::<OnEndScreen>)
        .configure_sets(
            Update,
            (
                MainMenuSet.run_if(in_state(GameState::Menu)),
                PlaySet.run_if(in_state(RoundState::In)),
                // (run_scored, freeze_inputs, freeze_sim).run_if(in_state(RoundState::Scored))
                // (countdown_round, freeze_inputs, freeze_sim).run_if(in_state(RoundState::Countdown))
            ),
        )
        .configure_sets(FixedUpdate, (PlaySet.run_if(in_state(RoundState::In)),))
        .add_event::<CollisionEvent>()
        .add_event::<ScoreEvent>();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_state::<GameState>()
        .init_state::<RoundState>()
        .add_systems(Startup, setup)
        .add_plugins(TheGamePlugin)
        .run();
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
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        assert!(arena_height > 0.);
        assert!(arena_width > 0.);

        match self {
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
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
        let arena_width = RIGHT_WALL - LEFT_WALL;
        assert!(arena_height > 0.);
        assert!(arena_width > 0.);

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

#[derive(Component)]
struct OnScoredScreen;

#[derive(Component)]
struct OnEndScreen;

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);
