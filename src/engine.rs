use crate::board::{
    BoardIndex, Direction, GameError, PlayerBoard, REV_POSSIBLE_INNER_PITS,
    REV_POSSIBLE_OUTER_PITS, Row, SowResult, X_SIZE,
};
use crate::state::{Action, ApplyResult, GameState, TurnPhase};

enum Step {
    Sowing { pit: BoardIndex, dir: Direction },
    Capture { pit: BoardIndex, dir: Direction },
    NeedDirection { pit: BoardIndex },
    End,
}

pub fn apply(mut state: GameState, action: Action) -> Result<ApplyResult, GameError> {
    let cp = state.current_player;
    let opp = 1 - cp;

    let mut step = match (action, &state.phase) {
        (Action::Pit(idx), TurnPhase::SelectPit) => {
            let pit = BoardIndex::new(idx)?;
            if is_reverse_possible(pit, &state.boards[cp], &state.boards[opp]) {
                Step::NeedDirection { pit }
            } else {
                Step::Sowing { pit, dir: Direction::Forward }
            }
        }
        (Action::Direction(dir), TurnPhase::SelectDirection { pit }) => {
            Step::Sowing { pit: *pit, dir }
        }
        _ => return Err(GameError::InvalidMove),
    };

    loop {
        step = match step {
            Step::NeedDirection { pit } => {
                state.phase = TurnPhase::SelectDirection { pit };
                return Ok(ApplyResult::Ongoing(state));
            }
            Step::Sowing { pit, dir } => match state.boards[cp].sow(pit, dir) {
                Ok(SowResult::Continue { pit }) => {
                    if is_capture_possible(pit, &state.boards[cp], &state.boards[opp]) {
                        Step::Capture { pit, dir }
                    } else if is_reverse_possible(pit, &state.boards[cp], &state.boards[opp]) {
                        Step::NeedDirection { pit }
                    } else {
                        Step::Sowing { pit, dir }
                    }
                }
                Ok(SowResult::End) => Step::End,
                Err(e) => return Err(e),
            },
            Step::Capture { pit, dir } => {
                let (_, col) = pit.split();
                let mirror = BoardIndex::new(X_SIZE - col - 1).unwrap();
                let seeds = state.boards[opp].take_seeds(mirror);
                match state.boards[cp].capture(pit, dir, seeds) {
                    Ok(SowResult::Continue { pit }) => Step::Sowing { pit, dir },
                    Ok(SowResult::End) => Step::End,
                    Err(e) => unreachable!("Capture logic error: {:?}", e),
                }
            }
            Step::End => {
                let next = 1 - cp;
                if !state.boards[next].has_moves() {
                    return Ok(ApplyResult::GameOver { winner: cp });
                }
                state.current_player = next;
                state.phase = TurnPhase::SelectPit;
                return Ok(ApplyResult::Ongoing(state));
            }
        }
    }
}

fn is_reverse_possible(pit: BoardIndex, cp: &PlayerBoard, opp: &PlayerBoard) -> bool {
    let (row, col) = pit.split();

    match row {
        Row::Inner => {
            if !REV_POSSIBLE_INNER_PITS.contains(&col) {
                return false;
            }
        }
        Row::Outer => {
            if !REV_POSSIBLE_OUTER_PITS.contains(&col) {
                return false;
            }
        }
    }

    let seeds = cp[row][col];
    let mut capture_pit = pit;
    for _ in 0..seeds {
        capture_pit = capture_pit.step(Direction::Reverse);
    }

    let (row, col) = capture_pit.split();
    if row != Row::Inner {
        return false;
    }
    if cp[row][col] == 0 {
        return false;
    }
    if opp[Row::Inner][col] == 0 || opp[Row::Outer][col] == 0 {
        return false;
    }
    true
}

fn is_capture_possible(pit: BoardIndex, cp: &PlayerBoard, opp: &PlayerBoard) -> bool {
    let (row, col) = pit.split();
    if row != Row::Inner {
        return false;
    }
    if cp[row][col] == 0 {
        return false;
    }
    let col_mirror = X_SIZE - col - 1;
    opp[Row::Inner][col_mirror] > 0 && opp[Row::Outer][col_mirror] > 0
}
