use std::io;

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

    fn sow(&mut self, opponent: &mut Player, mut idx: usize) -> bool {
        loop {
            match self.sow_once(idx) {
                SowResult::Invalid => return false,
                SowResult::End     => return true,
                SowResult::Continue { pit: next } => {
                    if next < X_SIZE && self.is_capture_possible(opponent, next) {
                        self.capture(opponent, next);
                    }
                    idx = next;
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
            _ => SowResult::Continue { pit: curr_idx },
        }
    }
    
    fn capture(&mut self, opponent: &mut Player, idx: usize) {
        let opp_pit = Self::mirror_pit(idx);
        let captured = opponent.pits[opp_pit] + opponent.pits[opp_pit + X_SIZE];
        opponent.pits[opp_pit] = 0;
        opponent.pits[opp_pit + X_SIZE] = 0;

        self.pits[idx] += captured;
    }

    fn is_capture_possible(&self, opponent: &Player, pit: usize) -> bool {
        let opp_pit = Self::mirror_pit(pit);
        opponent.pits[opp_pit] > 0 && opponent.pits[opp_pit + X_SIZE] > 0
    }

    fn is_rev_possible(idx: usize) -> bool {
        REV_POSSIBLE_PITS.contains(&idx)
    }

    fn get_row(&self, row: Row) -> &[u8] {
        match row {
            Row::Inner => &self.pits[..X_SIZE],
            Row::Outer => &self.pits[X_SIZE..],
        }
    }

    fn mirror_pit(idx: usize) -> usize {
        X_SIZE - 1 - idx
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

struct Game {
    players: [Player; 2],
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
