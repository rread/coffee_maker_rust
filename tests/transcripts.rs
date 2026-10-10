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
    Game::new(&mut out, NoPause).run(input.as_slice()).unwrap();
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

#[test]
fn numbered_prompts() {
    let mut out = Vec::new();
    let mut game = Game::new(&mut out, NoPause).with_numbered_prompts();
    game.start_without_banner().unwrap();
    for line in ["look note", "", "take the mug now", "xyzzy"] {
        game.handle_line(line).unwrap();
    }
    let out = String::from_utf8(out).unwrap();
    let prompts: Vec<_> = out
        .lines()
        .filter_map(|l| l.strip_prefix("[")?.split_once("] ? ").map(|(n, _)| n))
        .collect();
    assert_eq!(prompts, ["1", "2", "3", "4", "5"]);
    assert!(out.ends_with("\n---\n[5] ? "), "{out}");
}
