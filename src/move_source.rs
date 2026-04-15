use std::io::Write;

use crate::board::{BoardIndex, Direction, GameError, PITS_CNT, PlayerBoard, Row};

pub trait MoveSource {
    fn pick_pit(&mut self, player: &PlayerBoard, opponent: &PlayerBoard) -> BoardIndex;
    fn pick_direction(
        &mut self,
        player: &PlayerBoard,
        opponent: &PlayerBoard,
    ) -> Result<Direction, GameError>;
}

pub struct RandomInput;
impl MoveSource for RandomInput {
    fn pick_pit(&mut self, _player: &PlayerBoard, _opponent: &PlayerBoard) -> BoardIndex {
        BoardIndex::new(rand::random_range(..PITS_CNT)).unwrap()
    }

    fn pick_direction(
        &mut self,
        _player: &PlayerBoard,
        _opponent: &PlayerBoard,
    ) -> Result<Direction, GameError> {
        if rand::random_bool(0.5) {
            Ok(Direction::Forward)
        } else {
            Ok(Direction::Reverse)
        }
    }
}

pub struct ConsoleInput;
impl ConsoleInput {
    fn print_board(&self, player: &PlayerBoard, opponent: &PlayerBoard) {
        /* Opponent */
        for val in opponent[Row::Outer].iter().rev() {
            print!("{:02} ", val);
        }
        println!();

        for val in opponent[Row::Inner].iter() {
            print!("{:02} ", val);
        }
        println!("\n────────────────────────");

        /* Player */
        for val in player[Row::Inner].iter().rev() {
            print!("{:02} ", val);
        }
        println!();

        for val in player[Row::Outer].iter() {
            print!("{:02} ", val);
        }
        println!();
    }
}

impl MoveSource for ConsoleInput {
    fn pick_pit(&mut self, player: &PlayerBoard, opponent: &PlayerBoard) -> BoardIndex {
        self.print_board(player, opponent);

        print!("Choose index [0..15]: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        BoardIndex::new(input.trim().parse().unwrap()).unwrap()
    }

    fn pick_direction(
        &mut self,
        _player: &PlayerBoard,
        _opponent: &PlayerBoard,
    ) -> Result<Direction, GameError> {
        print!("Reverse [y/n]: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "y" {
            return Ok(Direction::Reverse);
        } else if input.trim() == "n" {
            return Ok(Direction::Forward);
        }

        Err(GameError::InvalidMove)
    }
}
