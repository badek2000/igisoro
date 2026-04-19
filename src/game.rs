use crate::engine::apply;
use crate::move_source::MoveSource;
use crate::state::{Action, ApplyResult, GameState, TurnPhase};

pub struct Game {
    state: GameState,
    inputs: [Box<dyn MoveSource>; 2],
    pub winner: Option<usize>,
}

impl Game {
    pub fn new(input1: Box<dyn MoveSource>, input2: Box<dyn MoveSource>) -> Self {
        Game {
            state: GameState::new(),
            inputs: [input1, input2],
            winner: None,
        }
    }

    pub fn start(&mut self) -> usize {
        for _ in self.by_ref() {}
        self.winner.unwrap()
    }
}

impl Iterator for Game {
    type Item = GameState;

    fn next(&mut self) -> Option<GameState> {
        if self.winner.is_some() {
            return None;
        }
        let initial_player = self.state.current_player;
        loop {
            let action = match self.state.phase {
                TurnPhase::SelectPit => {
                    let cp = self.state.current_player;
                    let opp = 1 - cp;
                    let pit =
                        self.inputs[cp].pick_pit(&self.state.boards[cp], &self.state.boards[opp]);
                    Action::Pit(pit.value())
                }
                TurnPhase::SelectDirection { .. } => {
                    let cp = self.state.current_player;
                    let opp = 1 - cp;
                    match self.inputs[cp]
                        .pick_direction(&self.state.boards[cp], &self.state.boards[opp])
                    {
                        Ok(dir) => Action::Direction(dir),
                        Err(_) => continue,
                    }
                }
            };
            match apply(self.state.clone(), action) {
                Ok(ApplyResult::Ongoing(new_state)) => {
                    self.state = new_state;
                    if self.state.current_player != initial_player {
                        return Some(self.state.clone());
                    }
                }
                Ok(ApplyResult::GameOver { winner }) => {
                    self.winner = Some(winner);
                    return None;
                }
                Err(_) => {}
            }
        }
    }
}
