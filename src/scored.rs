use bevy::prelude::*;

use crate::{
    despawn_screen, match_::RoundState, spawn_timed_message, GameState, GameTimer, MatchInfo,
    ScoreEvent, Scores,
};

pub fn scored_plugin(app: &mut App) {
    app.add_systems(Update, run_scored.run_if(in_state(RoundState::Scored)))
        .add_systems(OnEnter(RoundState::Scored), setup_scored)
        .add_systems(OnExit(RoundState::Scored), despawn_screen::<OnScoredScreen>);
}

#[derive(Component, Clone)]
struct OnScoredScreen;

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
    scores: Res<Scores>,
) {
    if timer.tick(time.delta()).finished() {
        score_events.clear();
        match_.round_count += 1;

        println!("match {}/{}", match_.round_count, match_.rounds_total);

        if match_.round_count == match_.rounds_total {
            if scores.a >= scores.b + 2 || scores.b >= scores.a + 2 {
                next_state_round.set(RoundState::Out);
                next_state_game.set(GameState::End);
            } else {
                // spawn_timed_message(commands, "Must win by 2 points!", 1.0, OnEndScreen);
                match_.rounds_total += 1;
                next_state_round.set(RoundState::Countdown);
            }
        } else {
            next_state_round.set(RoundState::Countdown);
        }
    }
}
