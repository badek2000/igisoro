use crate::board::{
    BoardIndex, Direction, PlayerBoard, Row, SowResult,
    REV_POSSIBLE_INNER_PITS, REV_POSSIBLE_OUTER_PITS, X_SIZE,
};
use crate::move_source::MoveSource;

struct Player {
    board: PlayerBoard,
    input: Box<dyn MoveSource>,
}

enum TurnState {
    PickPit,
    PickDirection { pit: BoardIndex },
    Sowing { pit: BoardIndex, dir: Direction },
    Capture { pit: BoardIndex, dir: Direction },
    End,
}

enum TurnResult {
    Continue,
    NoMovesAvailable,
}

pub struct Game {
    players: [Player; 2]
}

impl Game {
    pub fn new(move_source_1: Box<dyn MoveSource>, move_source_2: Box<dyn MoveSource>) -> Self {
        Game {
            players: [
                Player {board: PlayerBoard::new(), input: move_source_1},
                Player {board: PlayerBoard::new(), input: move_source_2},
            ],
        }
    }

    pub fn start(&mut self) {
        let mut cp = 0;
        loop {
            match self.execute_turn(cp) {
                TurnResult::Continue => cp = 1 - cp,
                TurnResult::NoMovesAvailable => {
                    println!("Player {} wins!", 2 - cp);
                    return;
                }
            }
        }
    }

    fn execute_turn(&mut self, cp_idx: usize) -> TurnResult {
        let opp_idx = 1 - cp_idx;
        let mut state = TurnState::PickPit;

        if !self.players[cp_idx].board.has_moves() {
            return TurnResult::NoMovesAvailable;
        }

        // TODO: Extract functions from each 
        loop {
            state = match state {
                TurnState::PickPit => {
                    let pit = self.players[cp_idx].input.pick_pit(
                        &self.players[cp_idx].board,
                         &self.players[opp_idx].board
                    );
                    if Self::is_reverse_possible(pit, &self.players[cp_idx].board, &self.players[opp_idx].board) {
                        TurnState::PickDirection { pit }
                    } else {
                        TurnState::Sowing { pit, dir: Direction::Forward }
                    }
                },
                TurnState::PickDirection { pit } => {
                    match self.players[cp_idx].input.pick_direction(     
                        &self.players[cp_idx].board,
                        &self.players[opp_idx].board
                    ) {
                        Ok(dir) => TurnState::Sowing { pit, dir },
                        Err(_) => TurnState::PickDirection { pit },
                    }
                },
                TurnState::Sowing { pit, dir } => {
                    match self.players[cp_idx].board.sow(pit, dir) {
                        Ok(SowResult::Continue { pit }) => {
                            if Self::is_capture_possible(pit, &self.players[cp_idx].board, &self.players[opp_idx].board) {
                                TurnState::Capture { pit, dir }
                            } else if Self::is_reverse_possible(pit, &self.players[cp_idx].board, &self.players[opp_idx].board) {
                                TurnState::PickDirection { pit }
                            } else {
                                TurnState::Sowing { pit, dir }
                            }
                        },
                        Ok(SowResult::End) => TurnState::End,
                        Err(_) => TurnState::PickPit,
                    }
                },
                TurnState::Capture { pit, dir} => { 
                    let (_, col) = pit.split();
                    let mirror = BoardIndex::new(Self::mirror_col(col)).unwrap();
                    let seeds = self.players[opp_idx].board.take_seeds(mirror);
                    match self.players[cp_idx].board.capture(pit, dir, seeds) {
                        Ok(SowResult::Continue { pit }) => TurnState::Sowing { pit, dir },
                        Ok(SowResult::End) => TurnState::End,
                        Err(e) => unreachable!("Capture logic error: {:?}", e),
                    }
                },
                TurnState::End => { return TurnResult::Continue; },
            }
        }
    }

    fn is_reverse_possible(pit: BoardIndex, cp_idx: &PlayerBoard, opp: &PlayerBoard) -> bool {
        /* Is player at special pit? */
        let (row, col) = pit.split();

        match row {
            Row::Inner => { if !REV_POSSIBLE_INNER_PITS.contains(&col) {return false} },
            Row::Outer => { if !REV_POSSIBLE_OUTER_PITS.contains(&col) {return false} },
        }

        /* Is capture possible in opposite dir? */
        let seeds = cp_idx[row][col];
        let mut capture_pit: BoardIndex = pit;
        for _ in 0..seeds {capture_pit = capture_pit.step(Direction::Reverse)};

        let (row, col) = capture_pit.split();
        
        if row != Row::Inner {
            return false;
        }

        if cp_idx[row][col] == 0 {
            return false;
        }

        if opp[Row::Inner][col] == 0 || opp[Row::Outer][col] == 0 {
            return false;
        } 

        true
    }

    fn is_capture_possible(pit: BoardIndex, cp: &PlayerBoard, opp: &PlayerBoard) -> bool {
        let (row, col) = pit.split();
        if row != Row::Inner { return false; }
        if cp[row][col] == 0 { return false; } 
    
        let col_mirror = Self::mirror_col(col);
        opp[Row::Inner][col_mirror] > 0 &&
        opp[Row::Outer][col_mirror] > 0
    }

    fn mirror_col(col: usize) -> usize {
        X_SIZE - col - 1
    }
}