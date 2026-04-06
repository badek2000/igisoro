use std::thread::sleep;
use std::time::Duration;

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

struct Player {
    pits: [u8; X_SIZE * Y_SIZE],
}

enum Row {
    Inner,
    Outer, 
}

enum SowResult {
    Invalid,
    End,
    Continue {pit: usize},
}

impl Player {
    fn new() -> Self {
        let mut pits = [0u8; X_SIZE * Y_SIZE];
        pits[0..X_SIZE].fill(4);

        Player { pits }
    }

    fn is_capture_possible(&self, opponent: &Player, col: usize) -> bool {
        let opp_col = Self::mirror_col(col);
        opponent.pits[opp_col] > 0 && opponent.pits[opp_col + X_SIZE] > 0
    }

    fn is_rev_possible(&self, idx: usize) -> bool {
        REV_POSSIBLE_PITS.contains(&idx)
    }

    fn sow(&mut self, opponent: &mut Player, idx: usize) -> bool {
        let mut pit = idx;
        
        loop {
            match self.sow_once(pit) {
                SowResult::Invalid => return false,
                SowResult::End     => return true,
                SowResult::Continue { pit: next } => {
                    if next < X_SIZE && self.is_capture_possible(opponent, next) {
                        self.capture(opponent, next);
                    }
                    pit = next;
                }
            }
        }
    }

    fn sow_once(&mut self, idx: usize) -> SowResult {
        if self.pits[idx] <= 1 { return SowResult::Invalid; }

        let mut curr_idx = idx;
        let mut seeds_cnt = self.pits[idx];
        self.pits[idx] = 0;

        while seeds_cnt != 0 {
            curr_idx += 1;
            curr_idx %= PITS_CNT;
            seeds_cnt -= 1;

            self.pits[curr_idx] += 1;
        }

        match self.pits[curr_idx] {
            1 => SowResult::End,
            _ => SowResult::Continue { pit: (curr_idx) },
        }
    }

    fn get_row(&self, row: Row) -> &[u8] {
        match row {
            Row::Inner => &self.pits[..X_SIZE],
            Row::Outer => &self.pits[X_SIZE..],
        }
    }

    fn mirror_col(col: usize) -> usize {
        X_SIZE - 1 - col
    }

    fn capture(&mut self, opponent: &mut Player, idx: usize) {
        let opp_col = Self::mirror_col(idx);
        let captured = opponent.pits[opp_col] + opponent.pits[opp_col + X_SIZE];
        opponent.pits[opp_col] = 0;
        opponent.pits[opp_col + X_SIZE] = 0;

        self.pits[idx] += captured;
    }

    fn print(&self, mirror: bool) {
        let (row1, row2) = if mirror {
            (self.get_row(Row::Outer), self.get_row(Row::Inner))
        } else {
            (self.get_row(Row::Inner), self.get_row(Row::Outer))
        };

        for val in row1.iter().rev() {
            print!("{:2} ", val);
        }
        print!("\n");
        for val in row2 {
            print!("{:2} ", val);
        }
        print!("\n");
    }

    fn is_any_move_available(&self) -> bool {
        self.pits.iter().any(|&pit| pit > 1u8 )
    }
}

fn gen_move() -> usize {
    rand::random_range(..PITS_CNT)
}

fn main() {
    let mut player1 = Player::new();
    let mut player2 = Player::new();

    let mut i: usize = 0;
    loop {
        println!("Move #{:02}", i);
        
        if !player2.is_any_move_available() {
            println!("Player 1 wins!");
            return;
        }
        let mut res1 = player2.sow(&mut player1, gen_move());
        while !res1 { res1 = player2.sow(&mut player1, gen_move()); }

        println!("After P2:");
        player2.print(true);
        player1.print(false);

        if !player1.is_any_move_available() {
            println!("Player 2 wins!");
            return;
        }
        let mut res2 = player1.sow(&mut player2, gen_move());
        while !res2 { res2 = player1.sow(&mut player2, gen_move()); }

        println!("After P1:");
        player2.print(true);
        player1.print(false);

        sleep(Duration::from_millis(10));
        i += 1;
    }
}
