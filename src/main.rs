use std::ffi::os_str::Display;
use std::ops::{Index, IndexMut};
use std::fmt

const X_SIZE: usize   = 8;
const Y_SIZE: usize   = 2;
const PITS_CNT: usize = X_SIZE * Y_SIZE;

const REV_POSSIBLE_PITS: [usize; 4] = [1, 6, 8, 15];

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

struct Player {
    pits: [[u8; X_SIZE]; Y_SIZE],
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
        PlayerBoard { inner: [0; X_SIZE], outer: [0; X_SIZE] }
    }

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
        let mut current_seeds_cnt = self[row][col];
        self[row][col] = 0;

        while current_seeds_cnt != 0 {
            pos = pos.step(dir);
            let (row, col) = pos.split();

            self[row][col] += 1;
            current_seeds_cnt -= 1;
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
        return captured;
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

struct Game {
    players: [PlayerBoard; 2],
    round: usize,
    curr_player: usize,
}

impl Game {
    fn new() -> Self {
        Game {
            players: [Player::new(), Player::new()],
            round: 0,
            curr_player: 1,
        }
    }

    fn start(&mut self) {
        self.main_loop();
    }

    fn player_and_opponent(&mut self) -> (&mut Player, &mut Player) {
        let (a, b) = self.players.split_at_mut(1);
        if self.curr_player == 0 {
            (&mut a[0], &mut b[0])
        } else {
            (&mut b[0], &mut a[0])
        }
    }

    fn print_board(&self) {
        println!("    --- Round {} ---    ", self.round);
        self.players[1].print(true);
        println!("────────────────────────");
        self.players[0].print(false);
        println!();
    }

    fn main_loop(&mut self) {
        loop {
            self.print_board();

            let cp = self.curr_player;
            let (player, opponent) = self.player_and_opponent();

            if !player.is_any_move_available() {
                println!("Player {} wins!", 2 - cp);
                return;
            }

            loop {
                let mov = gen_move();
                println!("P{} picks pit {}", cp + 1, mov);
                if player.sow(opponent, mov) {
                    break;
                }
            }

            self.curr_player = 1 - self.curr_player;
            self.round += 1;
        }
    }

}

fn gen_move() -> usize {
    rand::random_range(..PITS_CNT)
}

fn main() {
    let mut game = Game::new();
    game.start();
}
