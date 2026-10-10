//! All the Things in the world, and the table types indexed by them.

use enum_map::{Enum, EnumMap};

/// Everything in the world: the avatar, rooms, the things in the rooms,
/// sentinels, and even verbs.
///
/// Verbs are Things so the parser can use a single dictionary for both
/// positions of a 2-word command. Declaration order matters: rooms and the
/// inventory list their contents in this order.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
#[allow(clippy::enum_variant_names, reason = "names follow the Ada original")]
pub enum Thing {
    NoSuchThing,
    NoSuchVerb,
    Unsupplied,
    Avatar,
    // Rooms
    /// Alias for the current room.
    HereRoom,
    KitchenRoom,
    DiningRoom,
    LibraryRoom,
    LoungeRoom,
    BasementRoom,
    SunRoom,
    // Verbs
    DownVerb,
    EastVerb,
    LookVerb,
    NorthVerb,
    SouthVerb,
    UpVerb,
    WalkVerb,
    WestVerb,
    DrinkVerb,
    DropVerb,
    InventoryVerb,
    LockVerb,
    /// Only for "make coffee".
    MakeVerb,
    ReadVerb,
    SearchVerb,
    TakeVerb,
    UnlockVerb,
    UseVerb,
    WashVerb,
    WipeVerb,
    // Things, some takeable, some not
    CleanFunnel,
    DirtyMug,
    GroundCoffee,
    Kettle,
    CleanMug,
    DirtyFunnel,
    Kitty,
    MugOfCoffee,
    Note,
    SackOfFoodstuffs,
}

/// A Boolean attribute of every Thing.
pub type Flags = EnumMap<Thing, bool>;

/// A string for every Thing, such as its name or description.
pub type Strings = EnumMap<Thing, &'static str>;

/// Maps a Thing to another Thing. The main use is location.
pub type ThingToThing = EnumMap<Thing, Thing>;
