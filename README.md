# coffee_maker_tables

A second Rust port of [coffee_maker_ada](../coffee_maker_ada), this one
keeping the original's table-based design. Compare with
[coffee_maker_rs](../coffee_maker_rs), which models the world with
separate `Room`/`Thing`/`Verb` types instead.

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

Same as `coffee_maker_rs`: the game logic, messages, wrapping and pacing
match the Ada original. The only differences are that whole lines are read
(no 100-character buffer) and end of input quits cleanly. Both ports pass
the same golden transcripts in `tests/transcripts/`.
