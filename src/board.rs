use std::fmt;
use std::ops::{Index, IndexMut};

pub(crate) const X_SIZE: usize = 8;
const Y_SIZE: usize = 2;
pub(crate) const PITS_CNT: usize = X_SIZE * Y_SIZE;

pub(crate) const REV_POSSIBLE_INNER_PITS: [usize; 2] = [1, 6];
pub(crate) const REV_POSSIBLE_OUTER_PITS: [usize; 2] = [0, 7];

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
pub enum GameError {
    InvalidPitIdx,
    InvalidMove,
    EmptyPit,
    Logic,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Inner,
    Outer,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Forward,
    Reverse,
}

pub(crate) enum SowResult {
    End,
    Continue { pit: BoardIndex },
}

enum MoveType {
    Sow,
    Capture,
}

#[derive(Clone, Copy, Debug)]
pub struct BoardIndex(usize);
impl BoardIndex {
    pub fn new(idx: usize) -> Result<Self, GameError> {
        if idx >= PITS_CNT {
            return Err(GameError::InvalidPitIdx);
        }
        Ok(BoardIndex(idx))
    }

    pub fn split(&self) -> (Row, usize) {
        if self.0 < X_SIZE {
            (Row::Inner, self.0)
        } else {
            (Row::Outer, self.0 % X_SIZE)
        }
    }

    pub fn step(self, dir: Direction) -> Self {
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

type BoardArr = [u8; X_SIZE];
pub struct PlayerBoard {
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

impl PlayerBoard {
    pub(crate) fn new() -> Self {
        PlayerBoard {
            inner: [4; X_SIZE],
            outer: [0; X_SIZE],
        }
    }

    // TODO: Unify capture and sow into one function
    pub(crate) fn capture(
        &mut self,
        idx: BoardIndex,
        dir: Direction,
        mut seeds: u8,
    ) -> Result<SowResult, GameError> {
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

    pub(crate) fn sow(&mut self, idx: BoardIndex, dir: Direction) -> Result<SowResult, GameError> {
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

    pub(crate) fn take_seeds(&mut self, idx: BoardIndex) -> u8 {
        let (_, col) = idx.split();
        let captured = self[Row::Inner][col] + self[Row::Outer][col];
        self[Row::Inner][col] = 0;
        self[Row::Outer][col] = 0;
        captured
    }

    pub(crate) fn has_moves(&self) -> bool {
        self.inner.iter().chain(self.outer.iter()).any(|&s| s > 1)
    }

    fn validate_move(&self, idx: BoardIndex, move_type: MoveType) -> Result<(), GameError> {
        let (row, col) = idx.split();
        match move_type {
            MoveType::Sow => {
                if self[row][col] <= 1 {
                    return Err(GameError::InvalidMove);
                }
            }
            MoveType::Capture => {
                if self[row][col] == 0 {
                    return Err(GameError::EmptyPit);
                }
            }
        }

        Ok(())
    }
}
