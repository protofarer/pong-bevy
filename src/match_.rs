use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_vector_shapes::{painter::ShapePainter, shapes::LinePainter};

use crate::{
    despawn_screen, spawn_timed_message, CollisionEvent, CollisionSounds, GameState, GameTimer,
    MatchInfo, RoundData, ScoreEvent, Scores, BALL_COLOR, BALL_RADIUS, BALL_START_POSITION,
    BALL_START_SPEED, BALL_START_VELOCITY, BOTTOM_WALL, GAP_BETWEEN_PADDLE_AND_GOAL, GOAL_COLOR,
    GOAL_THICKNESS, LEFT_WALL, PADDLE_COLOR, PADDLE_SIZE, PADDLE_SPEED, RIGHT_WALL,
    SCORE_A_POSITION, SCORE_B_POSITION, SCORE_FONT_SIZE, TEXT_COLOR, TOP_WALL, WALL_COLOR,
    WALL_THICKNESS,
};

pub fn match_plugin(app: &mut App) {
    app.init_state::<RoundState>()
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                check_for_collisions,
                play_collision_sound,
                process_score,
            )
                .chain()
                .in_set(PlaySet),
        )
        .add_systems(
            Update,
            (
                run_end.run_if(in_state(GameState::End)),
                (update_score_ui, bevy::window::close_on_esc, run_match).in_set(MatchSet),
            ),
        )
        .add_systems(OnEnter(GameState::Match), setup_match)
        .add_systems(OnEnter(GameState::End), setup_end)
        .add_systems(OnExit(GameState::Match), despawn_screen::<OnMatchView>)
        .add_systems(OnExit(GameState::End), despawn_screen::<OnEndScreen>)
        .configure_sets(
            Update,
            (
                PlaySet.run_if(in_state(RoundState::In)),
                MatchSet.run_if(in_state(GameState::Match)),
            ),
        )
        .configure_sets(FixedUpdate, (PlaySet.run_if(in_state(RoundState::In)),))
        .add_event::<CollisionEvent>()
        .add_event::<ScoreEvent>();
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoundState {
    #[default]
    Out,
    In,
    Scored,
    Countdown,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct PlaySet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct MatchSet;

#[derive(Component)]
pub enum Player {
    A,
    B,
}

#[derive(Component)]
pub struct Paddle;

#[derive(Component)]
pub struct Ball;

#[derive(Component, Deref, DerefMut, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Collider;

#[derive(Component)]
pub struct Wall;

#[derive(Bundle)]
pub struct WallBundle {
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

pub enum WallLocation {
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
pub struct Goal;

#[derive(Bundle)]
pub struct GoalBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    boundary_type: Goal,
}

pub enum GoalLocation {
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

#[derive(Component)]
pub struct ScoreboardUi(Player);

#[derive(Component)]
pub struct OnMatchView;

#[derive(Component, Clone)]
pub struct OnEndScreen;

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
        Velocity(BALL_START_VELOCITY),
        OnMatchView,
    ));

    // Scores
    // A
    commands.spawn((
        ScoreboardUi(Player::A),
        TextBundle::from_sections([TextSection::from_style(TextStyle {
            font_size: SCORE_FONT_SIZE,
            color: TEXT_COLOR,
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
            color: TEXT_COLOR,
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

    for &y in points.iter() {
        painter.line(Vec3::new(0., y, 0.), Vec3::new(0., y + dash_length, 0.));
    }
}

pub fn run_match(mut painter: ShapePainter) {
    draw_midline(painter);
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
    collider_query: Query<
        (&Transform, Option<&Goal>, Option<&Wall>, Option<&Paddle>),
        With<Collider>,
    >,
    mut collision_events: EventWriter<CollisionEvent>,
    mut score_events: EventWriter<ScoreEvent>,
    mut round_data: ResMut<RoundData>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    for (transform, goal, wall, paddle) in &collider_query {
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
            } else if paddle.is_some() {
                collision_events.send(CollisionEvent::Paddle);

                // Increase ball speed every 3 returns
                round_data.paddle_hit_count += 1;
                if round_data.paddle_hit_count % 3 == 0 {
                    *ball_velocity = Velocity(ball_velocity.0 * 1.03);
                    info!(
                        "Increase ball velocity due to paddle hit to {:?}",
                        ball_velocity
                    );
                }

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
                    // TODO calc new ball angle based on distance from paddle center
                    let relative_impact_length = (ball_transform.translation.y
                        - transform.translation.y)
                        / (PADDLE_SIZE.y / 2.);
                    info!("rel_impact_len {}", relative_impact_length);
                    let dy = match relative_impact_length {
                        k if k <= 0.25 => 0.,
                        k if k > 0.25 => 50.,
                        k if k < -0.25 => -50.,
                        _ => 0.,
                    };
                    info!("ball vel_y increased by: {}", dy);

                    ball_velocity.x = -ball_velocity.x;
                    ball_velocity.y += dy;
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

pub fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSounds>,
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
                scores.a += 1;
            }
            ScoreEvent::B => {
                scores.b += 1;
            }
        }

        next_state_round.set(RoundState::Scored);
    }
}

pub fn setup_end(commands: Commands, scores: Res<Scores>, mut match_: ResMut<MatchInfo>) {
    if scores.a > scores.b {
        spawn_timed_message(commands, "Player A wins the match!", 1.0, OnEndScreen);
    } else {
        spawn_timed_message(commands, "Player B wins the match!", 1.0, OnEndScreen);
    }
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
