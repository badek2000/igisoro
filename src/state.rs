use crate::board::{BoardIndex, Direction, PlayerBoard};

#[derive(Clone)]
pub struct GameState {
    pub boards: [PlayerBoard; 2],
    pub current_player: usize,
    pub phase: TurnPhase,
}

#[derive(Clone)]
pub enum TurnPhase {
    SelectPit,
    SelectDirection { pit: BoardIndex },
}

pub enum Action {
    Pit(usize),
    Direction(Direction),
}

pub enum ApplyResult {
    Ongoing(GameState),
    GameOver { winner: usize },
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            boards: [PlayerBoard::new(), PlayerBoard::new()],
            current_player: 0,
            phase: TurnPhase::SelectPit,
        }
    }
}
