use std::time::Duration;

use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::camera::ScalingMode,
    sprite::MaterialMesh2dBundle,
};
use bevy_vector_shapes::prelude::*;

use crate::{
    GameState, GameTimer, BALL_COLOR, BALL_RADIUS, BALL_START_POSITION, BALL_START_SPEED,
    BOTTOM_WALL, GAP_BETWEEN_PADDLE_AND_GOAL, LEFT_WALL, PADDLE_A_START_POSITION,
    PADDLE_B_START_POSITION, PADDLE_COLOR, PADDLE_SIZE, PADDLE_SPEED, RIGHT_WALL, ROUNDS_TOTAL,
    SCORE_A_POSITION, SCORE_B_POSITION, SCORE_FONT_SIZE, TEXT_COLOR, TOP_WALL, WALL_THICKNESS,
};

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    commands.spawn(Camera2dBundle::default());

    let wall_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    let paddle_collision_sound = asset_server.load("sounds/med_shoot.wav");
    let goal_collision_sound = asset_server.load("sounds/jump.wav");
    commands.insert_resource(CollisionSound {
        wall: wall_collision_sound,
        paddle: paddle_collision_sound,
        goal: goal_collision_sound,
    });
    commands.insert_resource(Scores { a: 0, b: 0 });
    commands.insert_resource(MatchInfo {
        round_count: 0,
        rounds_total: ROUNDS_TOTAL,
    });
    commands.insert_resource(RoundData {
        paddle_hit_count: 0,
    });

    next_state.set(GameState::Menu);
}

#[derive(Event)]
pub enum CollisionEvent {
    Wall,
    Paddle,
    Goal,
}

#[derive(Event)]
pub enum ScoreEvent {
    A,
    B,
}

#[derive(Resource)]
pub struct CollisionSound {
    pub wall: Handle<AudioSource>,
    pub paddle: Handle<AudioSource>,
    pub goal: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct Scores {
    pub a: usize,
    pub b: usize,
}

#[derive(Resource)]
pub struct MatchInfo {
    pub round_count: usize,
    pub rounds_total: usize,
}

#[derive(Resource)]
pub struct RoundData {
    pub paddle_hit_count: usize,
}

pub fn spawn_timed_message(
    mut commands: Commands,
    msg: &str,
    duration: f32,
    marker: impl Component + Clone,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect {
                        top: Val::Px(50.),
                        ..default()
                    },
                    // align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            marker,
        ))
        .with_children(|parent| {
            parent
                .spawn((NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    // background_color: Color::GRAY.into(),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((TextBundle::from_section(
                        msg,
                        TextStyle {
                            font_size: 40.0,
                            color: TEXT_COLOR,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    }),));
                });
        });
    commands.insert_resource(GameTimer(Timer::from_seconds(duration, TimerMode::Once)));
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        // println!("despawning entity {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}
