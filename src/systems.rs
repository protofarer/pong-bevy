use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use crate::{
    Ball, Collider, CollisionEvent, CollisionSound, GameState, Goal, GoalBundle, GoalLocation,
    Match, Paddle, Player, ScoreEvent, ScoreboardUi, Scores, Velocity, Wall, WallBundle,
    WallLocation, BALL_COLOR, BALL_R, BALL_START_POSITION, BALL_START_SPEED, BOTTOM_WALL,
    GAP_BETWEEN_PADDLE_AND_BACKWALL, LEFT_WALL, PADDLE_A_START_VEC, PADDLE_B_START_VEC,
    PADDLE_COLOR, PADDLE_SIZE, PADDLE_SPEED, RIGHT_WALL, ROUNDS_TOTAL, SCORE_COLOR,
    SCORE_FONT_SIZE, TOP_WALL, WALL_THICKNESS,
};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
    commands.insert_resource(Match {
        round_count: 0,
        rounds_total: ROUNDS_TOTAL,
    });

    // Paddle A
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                // translation: PADDLE_A_START_VEC,
                translation: Vec3::new(LEFT_WALL + GAP_BETWEEN_PADDLE_AND_BACKWALL, 0., 0.),
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
    ));

    // Paddle B
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                // translation: PADDLE_B_START_VEC,
                translation: Vec3::new(RIGHT_WALL - GAP_BETWEEN_PADDLE_AND_BACKWALL, 0., 0.),
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
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(BALL_COLOR),
            transform: Transform::from_translation(BALL_START_POSITION)
                .with_scale(Vec2::splat(BALL_R * 2.).extend(1.)),
            ..default()
        },
        Ball,
        Velocity(rand_ball_dir() * BALL_START_SPEED),
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
            position_type: PositionType::Relative,
            top: Val::Percent(10.),
            left: Val::Percent(20.),
            ..default()
        }),
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
            position_type: PositionType::Relative,
            top: Val::Percent(10.),
            left: Val::Percent(80.),
            ..default()
        }),
    ));

    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));
    commands.spawn(GoalBundle::new(GoalLocation::Left));
    commands.spawn(GoalBundle::new(GoalLocation::Right));

    next_state.set(GameState::Match);
}

pub fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Player), With<Paddle>>,
    time: Res<Time>,
) {
    // ? &mut
    for (mut transform, player) in query.iter_mut() {
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

                let top_bound = TOP_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0;
                let bottom_bound = BOTTOM_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0;

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

                let top_bound = TOP_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0;
                let bottom_bound = BOTTOM_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0;

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
    mut commands: Commands,
    mut scores: ResMut<Scores>,
    mut ball_query: Query<(Entity, &mut Velocity, &Transform), With<Ball>>,
    mut collider_query: Query<(Entity, &Transform, Option<&Goal>, Option<&Wall>), With<(Collider)>>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut score_events: EventWriter<ScoreEvent>,
) {
    let (mut ball_entity, mut ball_velocity, ball_transform) = ball_query.single_mut();

    for (collider_entity, transform, goal, wall) in &collider_query {
        let collision = collide_with_side(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_R),
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
                        let mut a = scores.a;
                        a += 1;
                        score_events.send(ScoreEvent::A);
                    }
                    Collision::Left => {
                        let mut b = scores.b;
                        b += 1;
                        score_events.send(ScoreEvent::B);
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
                    _ => {}
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

fn rand_ball_dir() -> Vec2 {
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

pub fn update_scores(scores: Res<Scores>, mut query: Query<(&mut Text, &ScoreboardUi)>) {
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

pub fn setup_match(
    mut scores: ResMut<Scores>,
    mut match_: ResMut<Match>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // reset scores, round count
    let mut a = scores.a;
    a = 0;
    let mut b = scores.b;
    b = 0;
    let mut count = match_.round_count;
    count = 0;
    next_state.set(GameState::Round);
}

pub fn setup_round(
    mut commands: Commands,
    mut query: Query<(
        &mut Transform,
        &mut Velocity,
        Option<&Player>,
        Option<&Ball>,
        Option<&Paddle>,
    )>,
) {
    for (mut transform, mut velocity, player, ball, paddle) in query.iter_mut() {
        if ball.is_some() {
            // let (ball_entity, mut ball_transform, mut ball_velocity) = ball_query.single_mut();
            let mut translation = transform.translation;
            translation = BALL_START_POSITION;
            *velocity = Velocity(rand_ball_dir() * BALL_START_SPEED);
        } else if paddle.is_some() {
            if let Some(player) = player {
                let mut translation = transform.translation;
                match player {
                    Player::A => {
                        translation = PADDLE_A_START_VEC;
                    }
                    Player::B => {
                        translation = PADDLE_B_START_VEC;
                    }
                }
            }
        }
    }

    // // ? &mut
    // for (mut paddle_transform, paddle) in paddle_query.iter_mut() {
    //     let mut translation = paddle_transform.translation;
    //     match paddle.0 {
    //         Player::A => {
    //             translation = PADDLE_A_START_VEC;
    //         }
    //         Player::B => {
    //             translation = PADDLE_B_START_VEC;
    //         }
    //     }
    // }
}

pub fn end_round() {
    // no input
    // show scorer
    // pause for 3 sec
}

pub fn end_match() {
    // no input
    // show victor
    // pause for 3 sec
    // prompt for restart y/n
}
