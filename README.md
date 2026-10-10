# Coffee Maker in Rust

A Rust port of
[coffee_maker_ada](https://github.com/c3mnr0nz0yfgvxg4gi0sxhnwl9/coffee_maker_ada),
a tiny 2-word parser game, keeping the original's table-based design.

**Play it in your browser: <https://rread.github.io/coffee_maker_rust/>**

Or in a terminal:

```
cargo run --release
```

## Design

As in the Ada version, one `Thing` enum covers everything: the avatar,
rooms, objects, verbs, and the sentinels `NoSuchThing`, `NoSuchVerb` and
`Unsupplied`. Game state is a set of tables indexed by it, using
[`enum-map`](https://crates.io/crates/enum-map):

| Ada | Rust |
|---|---|
| `type Flags_Type is array(Thing_Id) of Boolean` | `type Flags = EnumMap<Thing, bool>` |
| `type Strings_Type is array(Thing_Id) of Unbounded_String` | `type Strings = EnumMap<Thing, &'static str>` |
| `type Thing_To_Thing_Type is array(Thing_Id) of Thing_Id` | `type ThingToThing = EnumMap<Thing, Thing>` |
| `(KITCHEN_ROOM => True, others => False)` | `enum_map! { KitchenRoom => true, _ => false }` |
| `Actions : array(Thing_Id) of access procedure` | `EnumMap<Thing, fn(&mut Game, &Command) -> io::Result<()>>` |
| `Words` array, linear search | `Vec<(&str, Thing)>`, linear search |

Location works the same way: a Thing located at `Avatar` is carried, and
one at `NoSuchThing` is nowhere. `src/tables.rs` holds every table's
initial contents; `src/game.rs` holds the actions, each following the Ada
procedure of the same name.

## Fidelity

The terminal version's game logic, messages (typos included), wrapping and
pacing match the Ada original. The only differences are that whole lines
are read (no 100-character buffer) and end of input quits cleanly. Golden
transcripts in `tests/transcripts/`, derived from the Ada source, check
this byte for byte.

The browser version drops the opening banner (`Begin`, the dictionary
size, `***`), numbers the prompt (`[1] ? `), and on screens too narrow for
80 columns (phones) wraps paragraphs to fit, using the same rule. The game
itself is the same.

## Playing in a browser

`web/` is a WebAssembly front end that prints the game onto green-bar
printer paper, keeping the original pacing. It drives the same library
through `Game::start` and `Game::handle_line`; the game's pauses become
browser timers.

It needs the `wasm32-unknown-unknown` target and
[trunk](https://trunkrs.dev):

```
cd web
trunk serve --open                         # play locally
trunk build --release --public-url ./      # static site in web/dist/
```

`web/dist/` is a self-contained static site (HTML, JS, and a ~80 KB
`.wasm`) that can be hosted anywhere, including GitHub Pages. The relative
`--public-url` makes it work under any repository path.

## License

GPL-3.0, the same as the Ada original this is ported from. See
[LICENSE](LICENSE).
