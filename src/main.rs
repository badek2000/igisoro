use igisoro::{ConsoleInput, Game, RandomInput};

fn main() {
    let mut game = Game::new(Box::new(ConsoleInput), Box::new(RandomInput));
    game.start();
}
