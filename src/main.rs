use std::io;
use std::process::ExitCode;

use coffee_maker_tables::{Game, Sleep};

fn main() -> ExitCode {
    let mut game = Game::new(io::stdout().lock(), Sleep);
    match game.run(io::stdin().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("coffee_maker: {e}");
            ExitCode::FAILURE
        }
    }
}
