//! Golden transcripts: each `tests/transcripts/NAME.in` is fed to the game
//! and the output must match `NAME.out` byte for byte. The `.out` files were
//! derived from the Ada source, not from this program.

use std::fs;
use std::path::Path;

use coffee_maker_tables::{Game, NoPause};

fn check(name: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/transcripts");
    let input = fs::read(dir.join(format!("{name}.in"))).unwrap();
    let expected = fs::read(dir.join(format!("{name}.out"))).unwrap();
    let mut out = Vec::new();
    Game::new(input.as_slice(), &mut out, NoPause)
        .run()
        .unwrap();
    if out != expected {
        let actual = dir.join(format!("{name}.actual"));
        fs::write(&actual, &out).unwrap();
        panic!(
            "{name}: output differs; compare with `diff {} {}`",
            dir.join(format!("{name}.out")).display(),
            actual.display()
        );
    }
}

macro_rules! transcripts {
    ($($name:ident),* $(,)?) => {
        $( #[test] fn $name() { check(stringify!($name)); } )*
    };
}

transcripts!(
    walkthrough,
    errors,
    navigation,
    inventory,
    make,
    wash_search,
    wrap
);
