use std::time::Duration;

use bevy::prelude::*;

use crate::{
    despawn_screen, rand_ball_dir, Ball, Paddle, Player, RoundState, Velocity, BALL_START_POSITION,
    BALL_START_SPEED, PADDLE_A_START_POSITION, PADDLE_B_START_POSITION, TEXT_COLOR,
};

pub fn countdown_plugin(app: &mut App) {
    app.add_systems(
        Update,
        run_countdown.run_if(in_state(RoundState::Countdown)),
    )
    .add_systems(OnEnter(RoundState::Countdown), setup_countdown)
    .add_systems(
        OnExit(RoundState::Countdown),
        move |to_despawn: Query<Entity, With<OnCountdownScreen>>, cmd: Commands| {
            despawn_screen::<OnCountdownScreen>(to_despawn, cmd);
        },
    );
}

#[derive(Component)]
pub struct OnCountdownScreen;

#[derive(Component)]
pub struct CountdownTimedMessage {
    timer: Timer,
    texts: [String; 4],
    cursor: usize,
}

pub fn setup_countdown(
    mut q_ball: Query<(&mut Transform, &mut Velocity), (With<Ball>, Without<Paddle>)>,
    mut q_paddle: Query<(&mut Transform, &Player), With<Paddle>>,
    mut commands: Commands,
) {
    let (mut ball_transform, mut ball_velocity) = q_ball.single_mut();
    ball_transform.translation = BALL_START_POSITION;
    *ball_velocity = Velocity(rand_ball_dir() * BALL_START_SPEED);

    for (mut paddle_transform, player) in q_paddle.iter_mut() {
        match player {
            Player::A => {
                paddle_transform.translation = PADDLE_A_START_POSITION;
            }
            Player::B => {
                paddle_transform.translation = PADDLE_B_START_POSITION;
            }
        }
    }

    let texts: [String; 4] = [
        "3..".to_string(),
        "2..".to_string(),
        "1..".to_string(),
        "Go!".to_string(),
    ];

    let init_text = texts[0].clone();

    commands.spawn((
        CountdownTimedMessage {
            timer: Timer::new(Duration::from_millis(300), TimerMode::Repeating),
            texts,
            cursor: 0,
        },
        OnCountdownScreen,
    ));

    let base_id = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    // align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnCountdownScreen,
        ))
        .id();
    commands.entity(base_id).with_children(|parent| {
        parent
            .spawn((NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect {
                        top: Val::Px(50.),
                        ..default()
                    },
                    ..default()
                },
                // background_color: Color::GRAY.into(),
                ..default()
            },))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        init_text,
                        TextStyle {
                            font_size: 40.0,
                            color: TEXT_COLOR,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    }),
                    OnCountdownScreen,
                ));
            });
    });
}

pub fn run_countdown(
    mut q_countdown_text: Query<&mut Text, With<OnCountdownScreen>>,
    mut q_countdown_tmsg: Query<&mut CountdownTimedMessage>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<RoundState>>,
) {
    let mut countdowner = q_countdown_tmsg.single_mut();

    countdowner.timer.tick(time.delta());

    if countdowner.timer.finished() {
        info!("countdowner finished!");
        countdowner.cursor += 1;
        if countdowner.cursor == 3 {
            next_state.set(RoundState::In);
            countdowner.cursor = 0;
            return;
        }

        let result = q_countdown_text.get_single_mut();
        if let Ok(mut text) = result {
            text.sections[0].value = countdowner.texts[countdowner.cursor].clone();
        }
    }
}
