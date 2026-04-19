use igisoro::{Action, ApplyResult, GameState, PlayerBoard, Row, X_SIZE, apply};

fn boards(
    p1_inner: [u8; X_SIZE],
    p1_outer: [u8; X_SIZE],
    p2_inner: [u8; X_SIZE],
    p2_outer: [u8; X_SIZE],
) -> GameState {
    let mut state = GameState::new();
    state.boards[0] = PlayerBoard::from_arrays(p1_inner, p1_outer);
    state.boards[1] = PlayerBoard::from_arrays(p2_inner, p2_outer);
    state
}

#[test]
fn sow_moves_seeds_forward() {
    let state = boards(
        [2, 0, 0, 0, 0, 0, 0, 0],
        [0; X_SIZE],
        [4; X_SIZE],
        [0; X_SIZE],
    );
    let result = apply(state, Action::Pit(0)).unwrap();
    let next = match result {
        ApplyResult::Ongoing(s) => s,
        ApplyResult::GameOver { .. } => panic!("unexpected game over"),
    };
    assert_eq!(next.boards[0][Row::Inner][0], 0);
    assert_eq!(next.boards[0][Row::Inner][1], 1);
    assert_eq!(next.boards[0][Row::Inner][2], 1);
    assert_eq!(next.current_player, 1);
}

#[test]
fn invalid_pit_returns_error() {
    let state = GameState::new();
    // Pit 16 is out of range
    assert!(apply(state, Action::Pit(16)).is_err());
}

#[test]
fn empty_pit_returns_error() {
    let state = boards(
        [0, 4, 4, 4, 4, 4, 4, 4],
        [0; X_SIZE],
        [4; X_SIZE],
        [0; X_SIZE],
    );
    // Pit 0 is empty — sow() returns InvalidMove
    assert!(apply(state, Action::Pit(0)).is_err());
}

#[test]
fn no_moves_triggers_game_over() {
    let state = boards(
        [2, 0, 0, 0, 0, 0, 0, 0],
        [0; X_SIZE],
        [1, 0, 0, 0, 0, 0, 0, 0], // no moves (all ≤ 1)
        [0; X_SIZE],
    );
    let result = apply(state, Action::Pit(0)).unwrap();
    assert!(matches!(result, ApplyResult::GameOver { winner: 0 }));
}

#[test]
fn capture_takes_opponent_seeds() {
    let state = boards(
        [3, 0, 0, 2, 0, 0, 0, 0],
        [0; X_SIZE],
        [0, 0, 0, 0, 2, 4, 4, 4],
        [0, 0, 0, 0, 2, 0, 0, 0],
    );

    let result = apply(state, Action::Pit(0)).unwrap();
    let next = match result {
        ApplyResult::Ongoing(s) => s,
        ApplyResult::GameOver { winner } => panic!("unexpected game over, winner={}", winner),
    };

    // Opponent col 4 should be emptied after capture
    assert_eq!(next.boards[1][Row::Inner][4], 0);
    assert_eq!(next.boards[1][Row::Outer][4], 0);
}
