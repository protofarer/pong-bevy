#![allow(unused)]
use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

const PADDLE_SIZE: Vec3 = Vec3::new(20., 150., 0.0);
const GAP_BETWEEN_PADDLE_AND_BACKWALL: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.;

const BALL_START_POSITION: Vec3 = Vec3::new(0., 0., 1.);
const BALL_R: f32 = 15.;
const BALL_START_SPEED: f32 = 400.;

const WALL_THICKNESS: f32 = 10.;
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const SCORE_A_POSITION: Vec3 = Vec3::new(-150., 200., 0.0);
const SCORE_B_POSITION: Vec3 = Vec3::new(150., 200., 0.0);

const SCORE_FONT_SIZE: f32 = 40.;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scores { a: 0, b: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                check_for_collisions,
                play_collision_sound,
            )
                .chain(),
        )
        .add_systems(Update, (update_scores, bevy::window::close_on_esc))
        .run();
}

enum Player {
    A,
    B,
}

#[derive(Component)]
struct Paddle(Player);

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
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
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
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
        }
    }
}

#[derive(Resource)]
struct Scores {
    a: usize,
    b: usize,
}

#[derive(Component)]
struct ScoreboardUi;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    let paddle_y = 0.;

    // Paddle A
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(LEFT_WALL + GAP_BETWEEN_PADDLE_AND_BACKWALL, paddle_y, 0.),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle(Player::A),
        Collider,
    ));

    // Paddle B
    // commands.spawn((
    //     SpriteBundle {
    //         transform: Transform {
    //             translation: Vec3::new(RIGHT_WALL - GAP_BETWEEN_PADDLE_AND_BACKWALL, paddle_y, 0.),
    //             scale: PADDLE_SIZE,
    //             ..default()
    //         },
    //         sprite: Sprite {
    //             color: PADDLE_COLOR,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     Paddle(Player::B),
    //     Collider,
    // ));

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
        Velocity(initial_ball_direction() * BALL_START_SPEED),
    ));

    // Score
    commands.spawn((
        ScoreboardUi,
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCORE_FONT_SIZE,
                    color: SCORE_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCORE_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.),
            left: Val::Px(5.),
            ..default()
        }),
    ));

    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));
}

fn initial_ball_direction() -> Vec2 {
    Vec2::new(-1., 0.)
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += 1.;
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        direction -= 1.;
    }

    let new_paddle_position =
        paddle_transform.translation.y + direction * PADDLE_SPEED * time.delta_seconds();

    let top_bound = TOP_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0;
    let bottom_bound = BOTTOM_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0;

    paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut scores: ResMut<Scores>,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    mut collider_query: Query<(Entity, &Transform), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    for (collider_entity, transform) in &collider_query {
        let collision = collide_with_side(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_R),
            Aabb2d::new(
                transform.translation.truncate(),
                transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            collision_events.send_default();

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.,
                Collision::Right => reflect_x = ball_velocity.x < 0.,
                Collision::Top => reflect_y = ball_velocity.y < 0.,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.,
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    // play sound once per frame if collision occurred
    if !collision_events.is_empty() {
        // this prevents events staying active next frame (since events last for 2 frames until auto-cleaned up by engine)
        collision_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

fn collide_with_side(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest = wall.closest_point(ball.center());
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

fn update_scores(scores: Res<Scores>, mut query: Query<&mut Text, With<ScoreboardUi>>) {
    let mut scores = query.single_mut();
}
