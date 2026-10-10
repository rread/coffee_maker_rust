//! The initial contents of every table. `_ =>` plays the part of Ada's
//! `others =>`.

use enum_map::enum_map;

use crate::thing::{Flags, Strings, Thing::*, ThingToThing};

pub fn name() -> Strings {
    enum_map! {
        Avatar => "you",
        BasementRoom => "in the basement",
        CleanFunnel => "a pour-over funnel that's ready to make coffee",
        CleanMug => "a large, clean coffee mug",
        DiningRoom => "the dining room",
        DropVerb => "the drop verb",
        DirtyFunnel => "a pour-over funnel that needs cleaning",
        GroundCoffee => "some ground coffee",
        InventoryVerb => "the inventory verb",
        Kettle => "an electric kettle",
        KitchenRoom => "in the kitchen",
        LoungeRoom => "in the lounge",
        MakeVerb => "the make verb",
        DirtyMug => "a large, dirty coffee mug",
        Note => "a hand-written note",
        NoSuchThing => "no such object",
        SackOfFoodstuffs => "a sack of drygoods from the grocery store",
        TakeVerb => "the take verb",
        WashVerb => "the wash verb",
        WipeVerb => "the wipe verb",
        _ => "<<noname>>",
    }
}

/// Descriptions, possibly long enough to wrap.
pub fn desc() -> Strings {
    enum_map! {
        Avatar => "a respectable person",
        BasementRoom => concat!(
            "You are in the basement.  It's dark, but there are no grues.  Such",
            " luck!  The room is large & unfinished.  The floor is cinderblock",
            " with a thick coat of dry dust.  The walls are brick & coated with",
            " nearly as much dust.  Several thick columns of brick prevent",
            " the ceiling from collapsing."
        ),
        CleanMug => concat!(
            "It's a coffee mug.  It's clean enough to drink from if there were",
            " coffee in it."
        ),
        DiningRoom => concat!(
            "You are in the dining room.  There's a wooden table in the center",
            " of the room.  The kitchen is to the south."
        ),
        DropVerb => concat!(
            "This is the object that implements the Drop action.  You should",
            " no be seeing this."
        ),
        InventoryVerb => concat!(
            "This is the object that implements the Inventory action.  You should",
            " no be seeing this."
        ),
        Kettle => concat!(
            "It's an electric kettle.  The barrel is glass.  Its lid is hinged.",
            "  It has a base with bumps & slots, looks like they fit into a",
            " custom stand."
        ),
        KitchenRoom => concat!(
            "You are in a huge kitchen with red-brick walls.  It has ovens",
            " embedded into the walls, three on each side.  There's a table",
            " near the center of the room.  On the table are a stand for an",
            " electric kettle & a note next to it.  There are open doorways",
            " to each of the 4 main compass directions (north, east, south",
            " west)."
        ),
        LibraryRoom => concat!(
            "You are in the library.  Tall bookcases line the walls.  There's a",
            " ladder on casters & attached to a rail that runs around the room",
            " so that people (including you) can reach even the top shelves. ",
            " Good thing, that, as those shelves are too high for even the",
            " tallest person you've ever met to reach.  The kitchen is to the",
            " west."
        ),
        LoungeRoom => concat!(
            "You are in the lounge.  A large window on the west lets in sunlight",
            " on bad days, twilight or moonlight when the weather is good.  It's",
            " furnished with comfy chairs & a comfy couch, perfect for people to",
            " take a seat with their coffee to talk or read quietly."
        ),
        MakeVerb => concat!(
            "This is the object that implements the Make action.  You should",
            " not be seeing this."
        ),
        Note => concat!(
            "In an admirably controlled hand, someone has written",
            " \"To make coffee, you will need the electric kettle, a mug,",
            " a pour-over funnel with built-in filter, & of course some ground",
            " coffee.\""
        ),
        SackOfFoodstuffs => concat!(
            "It's a paper sack, like from a grocery story.  It's stuffed full",
            " of drygoods (stuff that you can put on a shelf for weeks or",
            " longer without it going bad).  You feel a temptation to SEARCH",
            " this sack."
        ),
        SunRoom => concat!(
            "You walk up the stairs & into a comfortably welcoming room, perfect",
            " for sipping coffee with the kitty & a friend.  Congratulations,",
            " you've completed the game!"
        ),
        TakeVerb => concat!(
            "This is the object that implements the Take action.  You should",
            " no be seeing this."
        ),
        WashVerb => concat!(
            "This is the object that implements the Wash action.  You should",
            " not be seeing this."
        ),
        WipeVerb => concat!(
            "This is the object that implements the Wipe action.  You should",
            " not be seeing this."
        ),
        _ => "<<no desc>>",
    }
}

/// Where everything is. Rooms and verbs aren't anywhere, so they're at
/// `NoSuchThing`; a Thing located at `Avatar` is being carried.
pub fn location() -> ThingToThing {
    enum_map! {
        Avatar => KitchenRoom,
        Kettle => DiningRoom,
        DirtyFunnel => LibraryRoom,
        DirtyMug => LoungeRoom,
        Note => KitchenRoom,
        SackOfFoodstuffs => BasementRoom,
        _ => NoSuchThing,
    }
}

/// Rooms the avatar has entered.
pub fn here_before() -> Flags {
    enum_map! {
        KitchenRoom => true,
        _ => false,
    }
}

/// Most Things, including rooms and verbs, can't be taken. A Thing's flag
/// may change during play.
pub fn takeable() -> Flags {
    enum_map! {
        CleanMug | DirtyMug | CleanFunnel | DirtyFunnel | GroundCoffee | Kettle
        | SackOfFoodstuffs => true,
        _ => false,
    }
}

/// Droppable by default; that's harmless for things that can't be taken.
pub fn dropable() -> Flags {
    enum_map! {
        MugOfCoffee => false,
        _ => true,
    }
}

/// Rooms emit light unless marked otherwise.
pub fn illuminated() -> Flags {
    enum_map! { _ => true }
}

/// Few items emit light (none, in this game).
pub fn emits_light() -> Flags {
    enum_map! { _ => false }
}

// The map. `NoSuchThing` means you can't go that way.

pub fn targets_north() -> ThingToThing {
    enum_map! {
        KitchenRoom => DiningRoom,
        LoungeRoom => KitchenRoom,
        _ => NoSuchThing,
    }
}

pub fn targets_east() -> ThingToThing {
    enum_map! {
        KitchenRoom => LibraryRoom,
        LoungeRoom => KitchenRoom,
        _ => NoSuchThing,
    }
}

pub fn targets_south() -> ThingToThing {
    enum_map! {
        DiningRoom => KitchenRoom,
        KitchenRoom => LoungeRoom,
        _ => NoSuchThing,
    }
}

pub fn targets_west() -> ThingToThing {
    enum_map! {
        KitchenRoom => LoungeRoom,
        LibraryRoom => KitchenRoom,
        _ => NoSuchThing,
    }
}

pub fn targets_up() -> ThingToThing {
    enum_map! {
        BasementRoom => KitchenRoom,
        _ => NoSuchThing,
    }
}

pub fn targets_down() -> ThingToThing {
    enum_map! {
        KitchenRoom => BasementRoom,
        _ => NoSuchThing,
    }
}

/// The dictionary. One table serves both verbs and objects; the parser
/// tells them apart only by position.
pub const WORDS: [(&str, crate::thing::Thing); 55] = [
    ("", Unsupplied),
    ("bag", SackOfFoodstuffs),
    ("cat", Kitty),
    ("coffee", GroundCoffee),
    ("d", DownVerb),
    ("down", DownVerb),
    ("drink", DrinkVerb),
    ("drop", DropVerb),
    ("drygoods", SackOfFoodstuffs),
    ("e", EastVerb),
    ("east", EastVerb),
    ("electric", Kettle),
    ("exam", LookVerb),
    ("examine", LookVerb),
    ("filter", DirtyFunnel),
    ("food", SackOfFoodstuffs),
    ("foodstuffs", SackOfFoodstuffs),
    ("funnel", DirtyFunnel),
    ("get", TakeVerb),
    ("go", WalkVerb),
    ("groceries", SackOfFoodstuffs),
    ("grounds", GroundCoffee),
    ("here", HereRoom),
    ("i", InventoryVerb),
    ("invent", InventoryVerb),
    ("inventory", InventoryVerb),
    ("kettle", Kettle),
    ("kitten", Kitty),
    ("kitty", Kitty),
    ("l", LookVerb),
    ("lock", LockVerb),
    ("look", LookVerb),
    ("make", MakeVerb),
    ("me", Avatar),
    ("mug", DirtyMug),
    ("n", NorthVerb),
    ("north", NorthVerb),
    ("note", Note),
    ("read", ReadVerb),
    ("rinse", WashVerb),
    ("s", SouthVerb),
    ("sack", SackOfFoodstuffs),
    ("search", SearchVerb),
    ("self", Avatar),
    ("south", SouthVerb),
    ("take", TakeVerb),
    ("u", UpVerb),
    ("unlock", UnlockVerb),
    ("up", UpVerb),
    ("use", UseVerb),
    ("w", WestVerb),
    ("walk", WalkVerb),
    ("wash", WashVerb),
    ("wipe", WipeVerb),
    ("west", WestVerb),
];
