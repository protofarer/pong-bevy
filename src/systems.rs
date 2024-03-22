use std::time::Duration;

use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::camera::ScalingMode,
    sprite::MaterialMesh2dBundle,
};
use bevy_vector_shapes::prelude::*;

use crate::{
    Ball, Collider, CollisionEvent, CollisionSound,  GameState, GameTimer,
    Goal, GoalBundle, GoalLocation, MatchInfo, OnEndScreen, OnMatchView,
    OnScoredScreen, Paddle, Player, RoundState, ScoreEvent, ScoreboardUi, Scores, Velocity, Wall,
    WallBundle, WallLocation, BALL_COLOR, BALL_RADIUS, BALL_START_POSITION, BALL_START_SPEED,
    BOTTOM_WALL, GAP_BETWEEN_PADDLE_AND_GOAL, LEFT_WALL, PADDLE_A_START_POSITION,
    PADDLE_B_START_POSITION, PADDLE_COLOR, PADDLE_SIZE, PADDLE_SPEED, RIGHT_WALL, ROUNDS_TOTAL,
    SCORE_A_POSITION, SCORE_B_POSITION, SCORE_COLOR, SCORE_FONT_SIZE, TEXT_COLOR, TOP_WALL,
    WALL_THICKNESS,
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

    next_state.set(GameState::Menu);
}

pub fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Player), With<Paddle>>,
    time: Res<Time>,
) {
    for (mut transform, player) in query.iter_mut() {
        let top_bound = TOP_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0;
        let bottom_bound = BOTTOM_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0;

        match player {
            Player::A => {
                let mut direction = 0.;

                if keyboard_input.pressed(KeyCode::KeyW) {
                    direction += 1.;
                }
                if keyboard_input.pressed(KeyCode::KeyS) {
                    direction -= 1.;
                }

                let new_paddle_position =
                    transform.translation.y + direction * PADDLE_SPEED * time.delta_seconds();

                transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
            }
            Player::B => {
                let mut direction = 0.;

                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    direction += 1.;
                }
                if keyboard_input.pressed(KeyCode::ArrowDown) {
                    direction -= 1.;
                }

                let new_paddle_position =
                    transform.translation.y + direction * PADDLE_SPEED * time.delta_seconds();

                transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
            }
        }
    }
}

pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

pub fn check_for_collisions(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(&Transform, Option<&Goal>, Option<&Wall>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut score_events: EventWriter<ScoreEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    for (transform, goal, wall) in &collider_query {
        let collision = collide_with_side(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_RADIUS),
            Aabb2d::new(
                transform.translation.truncate(),
                transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            if goal.is_some() {
                collision_events.send(CollisionEvent::Goal);

                match collision {
                    Collision::Right => {
                        score_events.send(ScoreEvent::B);
                    }
                    Collision::Left => {
                        score_events.send(ScoreEvent::A);
                    }
                    _ => {}
                }
            } else if wall.is_some() {
                collision_events.send(CollisionEvent::Wall);

                let mut reflect_y = false;

                match collision {
                    Collision::Top => reflect_y = ball_velocity.y < 0.,
                    Collision::Bottom => reflect_y = ball_velocity.y > 0.,
                    _ => {}
                }

                if reflect_y {
                    ball_velocity.y = -ball_velocity.y;
                }
            } else {
                collision_events.send(CollisionEvent::Paddle);

                let mut reflect_y = false;
                let mut reflect_x = false;

                match collision {
                    Collision::Left => reflect_x = ball_velocity.x > 0.,
                    Collision::Right => reflect_x = ball_velocity.x < 0.,
                    Collision::Top => reflect_y = ball_velocity.y < 0.,
                    Collision::Bottom => reflect_y = ball_velocity.y > 0.,
                }

                if reflect_y {
                    ball_velocity.y = -ball_velocity.y;
                }

                if reflect_x {
                    ball_velocity.x = -ball_velocity.x;
                }
            }
        }
    }
}

pub fn rand_ball_dir() -> Vec2 {
    // up-left hits paddle
    // Vec2::new(-0.5, 0.8)
    // down-left misses paddle
    Vec2::new(-0.5, -0.5)
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

pub fn collide_with_side(ball: BoundingCircle, boundary: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&boundary) {
        return None;
    }

    let closest = boundary.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

pub fn update_score_ui(scores: Res<Scores>, mut query: Query<(&mut Text, &ScoreboardUi)>) {
    for (mut score, scoreboard) in &mut query {
        match scoreboard.0 {
            Player::A => {
                score.sections[0].value = scores.a.to_string();
            }
            Player::B => {
                score.sections[0].value = scores.b.to_string();
            }
        }
    }
}

pub fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    // play sound once per frame if collision occurred
    for ev in collision_events.read() {
        match ev {
            CollisionEvent::Wall => {
                commands.spawn(AudioBundle {
                    source: sound.wall.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            CollisionEvent::Paddle => {
                commands.spawn(AudioBundle {
                    source: sound.paddle.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            CollisionEvent::Goal => {
                commands.spawn(AudioBundle {
                    source: sound.goal.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
        }
    }

    // ? not sure if this needed
    if !collision_events.is_empty() {
        // this prevents events staying active next frame (since events last for 2 frames until auto-cleaned up by engine)
        collision_events.clear();
    }
}

pub fn setup_menu(mut commands: Commands) {}

pub fn setup_match(
    mut scores: ResMut<Scores>,
    mut match_: ResMut<MatchInfo>,
    mut next_state: ResMut<NextState<RoundState>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("IN setup_match");
    scores.a = 0;
    scores.b = 0;
    match_.round_count = 0;

    // Paddle A
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(LEFT_WALL + GAP_BETWEEN_PADDLE_AND_GOAL, 0., 0.),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Player::A,
        Collider,
        OnMatchView,
    ));

    // Paddle B
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(RIGHT_WALL - GAP_BETWEEN_PADDLE_AND_GOAL, 0., 0.),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Player::B,
        Collider,
        OnMatchView,
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(BALL_COLOR),
            transform: Transform::from_translation(BALL_START_POSITION)
                .with_scale(Vec2::splat(BALL_RADIUS * 2.).extend(1.)),
            ..default()
        },
        Ball,
        Velocity(rand_ball_dir() * BALL_START_SPEED),
        OnMatchView,
    ));

    // Scores
    // A
    commands.spawn((
        ScoreboardUi(Player::A),
        TextBundle::from_sections([TextSection::from_style(TextStyle {
            font_size: SCORE_FONT_SIZE,
            color: SCORE_COLOR,
            ..default()
        })])
        .with_style(Style {
            // position_type: PositionType::Relative,
            // top: Val::Px(100.),
            // left: Val::Percent(25.),
            top: SCORE_A_POSITION.top,
            left: SCORE_A_POSITION.left,
            ..default()
        }),
        OnMatchView,
    ));
    // B
    commands.spawn((
        ScoreboardUi(Player::B),
        TextBundle::from_sections([TextSection::from_style(TextStyle {
            font_size: SCORE_FONT_SIZE,
            color: SCORE_COLOR,
            ..default()
        })])
        .with_style(Style {
            // position_type: PositionType::Relative,
            top: SCORE_B_POSITION.top,
            left: SCORE_B_POSITION.left,
            ..default()
        }),
        OnMatchView,
    ));

    commands.spawn((WallBundle::new(WallLocation::Bottom), OnMatchView));
    commands.spawn((WallBundle::new(WallLocation::Top), OnMatchView));
    commands.spawn((GoalBundle::new(GoalLocation::Left), OnMatchView));
    commands.spawn((GoalBundle::new(GoalLocation::Right), OnMatchView));

    next_state.set(RoundState::Countdown);
}

pub fn process_score(
    mut scores: ResMut<Scores>,
    mut next_state_round: ResMut<NextState<RoundState>>,
    mut score_events: EventReader<ScoreEvent>,
) {
    // single expected event pattern
    if !score_events.is_empty() {
        println!("score_event!",);
        let score_events: Vec<&ScoreEvent> = score_events.read().collect();

        match score_events[0] {
            ScoreEvent::A => {
                // let mut a = scores.a;
                // a += 1;
                scores.a += 1;
            }
            ScoreEvent::B => {
                // let mut b = scores.b;
                // b += 1;
                scores.b += 1;
            }
        }

        next_state_round.set(RoundState::Scored);
    }
}

// ? pass an instanced Bundle or Bundle Constructor that accepts a msg, aka the UI entity and its components
fn spawn_timed_message(
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

// pub fn tick() {
//     println!("tick",);
// }

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        // println!("despawning entity {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}

pub fn setup_end(commands: Commands) {
    // commands.insert_resource(GameTimer(Timer::from_seconds(5., TimerMode::Once)));
    spawn_timed_message(commands, "match fin!", 1.0, OnEndScreen);
}

pub fn run_end(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if timer.tick(time.delta()).finished() {
        info!("Match Ended. Auto-starting over");
        next_state.set(GameState::Menu);
    }
}

pub fn setup_scored(commands: Commands, mut score_events: EventReader<ScoreEvent>) {
    let scorer = score_events.read().collect::<Vec<&ScoreEvent>>()[0];
    let scorer_text = match scorer {
        ScoreEvent::A => "A",
        ScoreEvent::B => "B",
    };
    let message = format!("Player {} scores!", scorer_text);
    spawn_timed_message(commands, &message, 2.0, OnScoredScreen);
}

pub fn run_scored(
    mut next_state_round: ResMut<NextState<RoundState>>,
    mut next_state_game: ResMut<NextState<GameState>>,
    mut match_: ResMut<MatchInfo>,
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    mut score_events: EventReader<ScoreEvent>,
) {
    if timer.tick(time.delta()).finished() {
        score_events.clear();

        match_.round_count += 1;
        println!("match {}/{}", match_.round_count, match_.rounds_total);

        if match_.round_count == match_.rounds_total {
            // will use the Exit Gamestate::Match to display victory screen
            next_state_round.set(RoundState::Out);
            next_state_game.set(GameState::End);
        } else {
            next_state_round.set(RoundState::Countdown);
        }
    }
}

pub fn draw_midline(mut painter: ShapePainter) {
    let height = TOP_WALL - BOTTOM_WALL - WALL_THICKNESS;
    let width = 1.0;
    let n_dashes = 10.;
    let dash_length = height / (2. * n_dashes);
    let line_color = Color::WHITE;

    painter.thickness = width;
    painter.color = line_color;

    let mut points = Vec::new();
    let mut y = (-height / 2.) + (0.5 * dash_length);
    while y < height / 2. {
        points.push(y);
        y += dash_length * 2.;
    }

    // painter.line(
    //     (Vec3::new(0., line_length / 2., 0.)),
    //     Vec3::new(0., -line_length / 2., 0.),
    // );

    for &y in points.iter() {
        painter.line(Vec3::new(0., y, 0.), Vec3::new(0., y + dash_length, 0.));
    }
}

pub fn run_match(mut painter: ShapePainter) {
    draw_midline(painter);
}
