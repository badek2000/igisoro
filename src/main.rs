use std::ops::{Index, IndexMut};
use std::fmt;
use std::io::Write;

const X_SIZE: usize   = 8;
const Y_SIZE: usize   = 2;
const PITS_CNT: usize = X_SIZE * Y_SIZE;

const REV_POSSIBLE_INNER_PITS: [usize; 2] = [1, 6];
const REV_POSSIBLE_OUTER_PITS: [usize; 2] = [8, 15];

/* 
 *  ╔═══════════════════════════════╗
 *  ║          P L A Y E R          ║
 *  ╠───┬───┬───┬───┬───┬───┬───┬───╣
 *  ║ 7 │ 6 │ 5 │ 4 │ 3 │ 2 │ 1 │ 0 ║
 *  ║ ○ │ x │ ○ │ ○ │ ○ │ ○ │ x │ ○ ║
 *  ╠───┼───┼───┼───┼───┼───┼───┼───╣
 *  ║ x │ ○ │ ○ │ ○ │ ○ │ ○ │ ○ │ x ║
 *  ║ 8 │ 9 │10 │11 │12 │13 │14 │15 ║
 *  ╚───┴───┴───┴───┴───┴───┴───┴───╝
 */

#[derive(Debug)]
enum GameError {
    InvalidPitIdx,
    InvalidMove,
    EmptyPit,
    Logic,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Row {
    Inner = 0,
    Outer = 1,
}

enum SowResult {
    End,
    Continue {pit: BoardIndex},
}

type BoardArr = [u8; X_SIZE];
struct PlayerBoard {   
    inner: BoardArr,
    outer: BoardArr,
}

impl Index<Row> for PlayerBoard {
    type Output = BoardArr;

    fn index(&self, row: Row) -> &Self::Output {
        match row {
            Row::Inner => &self.inner,
            Row::Outer => &self.outer,
        }
    }
}

impl IndexMut<Row> for PlayerBoard {
    fn index_mut(&mut self, row: Row) -> &mut Self::Output {
        match row {
            Row::Inner => &mut self.inner,
            Row::Outer => &mut self.outer,
        }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Forward,
    Reverse,
}

#[derive(Clone, Copy)]
struct BoardIndex(usize);
impl BoardIndex {
    fn new(idx: usize) -> Result<Self, GameError> {
        if idx >= PITS_CNT {
            return Err(GameError::InvalidPitIdx);
        }
        Ok(BoardIndex(idx))
    }

    fn split(&self) -> (Row, usize) {
        if self.0 < X_SIZE {
            (Row::Inner, self.0)
        } else {
            (Row::Outer, self.0 % X_SIZE)
        }
    }

    fn step(self, dir: Direction) -> Self {
        match dir {
            Direction::Forward => self.next(),
            Direction::Reverse => self.prev(),
        }
    }

    fn next(self) -> Self {
        BoardIndex((self.0 + 1) % PITS_CNT)
    }

    fn prev(self) -> Self {
        BoardIndex((self.0 + PITS_CNT - 1) % PITS_CNT)
    }
}

impl fmt::Display for BoardIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (row, col) = self.split();
        match row {
            Row::Inner => write!(f, "I{}", col),
            Row::Outer => write!(f, "O{}", col),
        }
    }
}

enum MoveType {
    Sow,
    Capture,
}

impl PlayerBoard {
    fn new() -> Self {
        PlayerBoard { inner: [4; X_SIZE], outer: [0; X_SIZE] }
    }

    // TODO: Unify capture and sow into one function
    fn capture(&mut self, idx: BoardIndex, dir: Direction, mut seeds: u8) -> Result<SowResult, GameError> {
        self.validate_move(idx, MoveType::Capture)?;
        let mut pos = idx;
        while seeds != 0 {
            pos = pos.step(dir);
            let (row, col) = pos.split();
            self[row][col] += 1;
            seeds -= 1;
        }

        let (row, col) = pos.split();
        match self[row][col] {
            0 => Err(GameError::Logic),
            1 => Ok(SowResult::End),
            _ => Ok(SowResult::Continue { pit: pos }),
        }
    }

    fn sow(&mut self, idx: BoardIndex, dir: Direction) -> Result<SowResult, GameError> {
        self.validate_move(idx, MoveType::Sow)?;
        let mut pos = idx;

        let (row, col) = pos.split();
        let mut cp_idx_seeds_cnt = self[row][col];
        self[row][col] = 0;

        while cp_idx_seeds_cnt != 0 {
            pos = pos.step(dir);
            let (row, col) = pos.split();

            self[row][col] += 1;
            cp_idx_seeds_cnt -= 1;
        }

        let (row, col) = pos.split();
        match self[row][col] {
            0 => Err(GameError::Logic),
            1 => Ok(SowResult::End),
            _ => Ok(SowResult::Continue { pit: pos }),
        }
    }

    fn take_seeds(&mut self, idx: BoardIndex) -> u8 {
        let (_, col) = idx.split();
        let captured = self[Row::Inner][col] + self[Row::Outer][col];
        self[Row::Inner][col] = 0;
        self[Row::Outer][col] = 0;
        captured
    }

    fn validate_move(&self, idx: BoardIndex, move_type: MoveType) -> Result<(), GameError> {
        let (row, col) = idx.split();
        match move_type {
            MoveType::Sow => {
                if self[row][col] <= 1 {return Err(GameError::InvalidMove);}
            }
            MoveType::Capture => {
                if self[row][col] == 0 {return Err(GameError::EmptyPit);}
            }
        }

        Ok(())
    }
}

trait MoveSource {
    fn pick_pit(&mut self, player: &PlayerBoard, opponent: &PlayerBoard) -> BoardIndex;
    fn pick_direction(&mut self, player: &PlayerBoard, opponent: &PlayerBoard) -> Result<Direction, GameError>;
}

struct RandomInput;
impl MoveSource for RandomInput {
    fn pick_pit(&mut self, _player: &PlayerBoard, _opponent: &PlayerBoard) -> BoardIndex {
        BoardIndex(rand::random_range(..PITS_CNT))
    }

    fn pick_direction(&mut self, _player: &PlayerBoard, _opponent: &PlayerBoard) -> Result<Direction, GameError> {
        if rand::random_bool(0.5) {
            Ok(Direction::Forward)
        } else {
            Ok(Direction::Reverse)
        }
    }
}

struct ConsoleInput;
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
        BoardIndex(input.trim().parse().unwrap())
    }
    
    fn pick_direction(&mut self, _player: &PlayerBoard, _opponent: &PlayerBoard) -> Result<Direction, GameError> {
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

struct Game {
    players: [Player; 2]
}

impl Game {
    fn new(move_source_1: Box<dyn MoveSource>, move_source_2: Box<dyn MoveSource>) -> Self {
        Game {
            players: [
                Player {board: PlayerBoard::new(), input: move_source_1},
                Player {board: PlayerBoard::new(), input: move_source_2},
            ],
        }
    }

    fn start(&mut self) {
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
                    let dir = self.players[cp_idx].input.pick_direction(     
                        &self.players[cp_idx].board,
                        &self.players[opp_idx].board
                    ).unwrap();  // TODO: Retry logic
                    TurnState::Sowing { pit, dir }
                },
                TurnState::Sowing { pit, dir} => { 
                    if Self::is_capture_possible(pit, &self.players[cp_idx].board, &self.players[opp_idx].board) {
                        TurnState::Capture { pit, dir }
                    } else {
                        match self.players[cp_idx].board.sow(pit, dir).unwrap() {
                            SowResult::Continue { pit } => {
                                if Self::is_reverse_possible(pit, &self.players[cp_idx].board, &self.players[opp_idx].board) {
                                    TurnState::PickDirection { pit }
                                } else {
                                    TurnState::Sowing { pit, dir }
                                }
                            }
                            SowResult::End => TurnState::End
                        }
                    }
                },
                TurnState::Capture { pit, dir} => { todo!() },
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
        for _ in 0..seeds {capture_pit = capture_pit.prev()};

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

    fn is_capture_possible(pit: BoardIndex, cp_idx: &PlayerBoard, opp: &PlayerBoard) -> bool {
        false
    }
}

use std::env;
fn main() {
    let mut game = Game::new(
        Box::new(ConsoleInput),
        Box::new(RandomInput),
    );
 
    game.start();

    let args: Vec<String> = env::args().collect();
    println!("Hello from {}", args[0]);
}
