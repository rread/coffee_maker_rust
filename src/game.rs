//! Game state as tables indexed by `Thing`, the verb actions, and the
//! read–parse–act loop.

use std::io::{self, BufRead, Write};
use std::time::Duration;

use enum_map::{EnumMap, enum_map};

use crate::Pacer;
use crate::parser::{Command, TooManyWords, Words};
use crate::tables;
use crate::text::{self, wrap};
use crate::thing::{Flags, Strings, Thing, Thing::*, ThingToThing};

const LINE_PAUSE: Duration = Duration::from_millis(100);
const COMMAND_PAUSE: Duration = Duration::from_millis(500);

/// A procedure that acts on a Command.
type Action<W, P> = fn(&mut Game<W, P>, &Command) -> io::Result<()>;

pub struct Game<W, P> {
    out: W,
    pacer: P,

    name: Strings,
    desc: Strings,
    location: ThingToThing,
    here_before: Flags,
    takeable: Flags,
    dropable: Flags,
    illuminated: Flags,
    emits_light: Flags,
    targets_north: ThingToThing,
    targets_east: ThingToThing,
    targets_south: ThingToThing,
    targets_west: ThingToThing,
    targets_up: ThingToThing,
    targets_down: ThingToThing,
    words: Words,
    /// For each Thing, what to do when a Command uses it as the verb.
    actions: EnumMap<Thing, Action<W, P>>,
    /// The player reached the sun room.
    is_done: bool,
    /// `End` has been printed; no more input is accepted.
    finished: bool,
    /// Show the number of the next input line in the prompt, e.g. `[3] ? `.
    numbered_prompts: bool,
    /// Lines the player has entered so far.
    lines_entered: u32,
    /// Paragraphs are wrapped to lines of at most this many bytes.
    columns: usize,
}

impl<W: Write, P: Pacer> Game<W, P> {
    pub fn new(out: W, pacer: P) -> Self {
        Game {
            out,
            pacer,
            name: tables::name(),
            desc: tables::desc(),
            location: tables::location(),
            here_before: tables::here_before(),
            takeable: tables::takeable(),
            dropable: tables::dropable(),
            illuminated: tables::illuminated(),
            emits_light: tables::emits_light(),
            targets_north: tables::targets_north(),
            targets_east: tables::targets_east(),
            targets_south: tables::targets_south(),
            targets_west: tables::targets_west(),
            targets_up: tables::targets_up(),
            targets_down: tables::targets_down(),
            words: Words::new(),
            actions: enum_map! {
                DownVerb => Self::down_action,
                DrinkVerb => Self::drink_action,
                DropVerb => Self::drop_action,
                EastVerb => Self::east_action,
                InventoryVerb => Self::inventory_action,
                LookVerb => Self::look_action,
                MakeVerb => Self::make_action,
                NorthVerb => Self::north_action,
                ReadVerb => Self::read_action,
                SearchVerb => Self::search_action,
                SouthVerb => Self::south_action,
                TakeVerb => Self::take_action,
                Unsupplied => Self::unsupplied_action,
                UpVerb => Self::up_action,
                WalkVerb => Self::walk_action,
                WashVerb => Self::wash_action,
                WestVerb => Self::west_action,
                WipeVerb => Self::wipe_action,
                _ => Self::not_a_verb_action,
            },
            is_done: false,
            finished: false,
            numbered_prompts: false,
            lines_entered: 0,
            columns: text::COLUMNS,
        }
    }

    /// Number each prompt with the input line it's waiting for: `[1] ? `,
    /// `[2] ? `, and so on. The original prompt is just `? `.
    pub fn with_numbered_prompts(mut self) -> Self {
        self.numbered_prompts = true;
        self
    }

    /// Wrap paragraphs printed from now on to `columns` instead of the
    /// original 80, using the same rule.
    pub fn set_columns(&mut self, columns: usize) {
        self.columns = columns;
    }

    /// Play until the player wins, types `@quit`, or input ends.
    pub fn run(&mut self, mut input: impl BufRead) -> io::Result<()> {
        self.start()?;
        while !self.finished {
            match read_line(&mut input)? {
                Some(line) => self.handle_line(&line)?,
                None => self.end()?,
            }
        }
        Ok(())
    }

    /// Print the original's banner, then the first room and prompt.
    pub fn start(&mut self) -> io::Result<()> {
        writeln!(self.out, "Begin")?;
        writeln!(
            self.out,
            "There are {} words in the dictionary.",
            self.words.len()
        )?;
        writeln!(self.out, "***")?;
        self.start_without_banner()
    }

    /// Print the first room and prompt, skipping the banner.
    pub fn start_without_banner(&mut self) -> io::Result<()> {
        self.look_here()?;
        self.prompt()
    }

    /// Act on one line typed by the player, then prompt for the next, or
    /// print `End` if the game is over.
    pub fn handle_line(&mut self, line: &str) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        self.lines_entered += 1;
        if line == "@quit" {
            return self.end();
        }
        match self.words.parse(line) {
            Ok(c) => {
                self.pacer.pause(COMMAND_PAUSE);
                self.do_command(&c)?;
            }
            Err(TooManyWords) => {
                writeln!(self.out, "Nope!  I'm a 2-word parser.  Try again.")?;
            }
        }
        self.prompt()
    }

    /// True once `End` has been printed.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn prompt(&mut self) -> io::Result<()> {
        if self.is_done {
            return self.end();
        }
        writeln!(self.out, "---")?;
        if self.numbered_prompts {
            write!(self.out, "[{}] ? ", self.lines_entered + 1)?;
        } else {
            write!(self.out, "? ")?;
        }
        self.out.flush()
    }

    fn end(&mut self) -> io::Result<()> {
        self.finished = true;
        writeln!(self.out, "End")?;
        self.out.flush()
    }

    fn do_command(&mut self, c: &Command) -> io::Result<()> {
        let action = self.actions[c.verb_id];
        action(self, c)
    }

    //******************************************************************
    // Convenience predicates

    /// True if and only if the avatar is carrying the Thing.
    fn carries(&self, x: Thing) -> bool {
        self.location[x] == Avatar
    }

    /// True if the avatar holds the Thing or is in the same room as it.
    fn has_access_to(&self, x: Thing) -> bool {
        self.location[x] == Avatar || self.location[x] == self.location[Avatar]
    }

    /// True if the room is lit or the avatar carries something that emits
    /// light.
    fn is_illuminated(&self) -> bool {
        self.illuminated[self.location[Avatar]]
            || self
                .location
                .iter()
                .any(|(t, &loc)| loc == Avatar && self.emits_light[t])
    }

    //******************************************************************
    // Output

    /// Print a word-wrapped paragraph, pausing after each line.
    fn paragraph(&mut self, s: &str) -> io::Result<()> {
        if s.is_empty() {
            return writeln!(self.out);
        }
        for line in wrap(s, self.columns) {
            self.out.write_all(line)?;
            self.out.write_all(b"\n")?;
            self.out.flush()?;
            self.pacer.pause(LINE_PAUSE);
        }
        Ok(())
    }

    //******************************************************************
    // Commonly used parts of verbs

    fn look_here(&mut self) -> io::Result<()> {
        let here = self.location[Avatar];
        self.paragraph(self.desc[here])?;
        let seen: Vec<&str> = self
            .location
            .iter()
            .filter(|&(t, &loc)| t != Avatar && loc == here)
            .map(|(t, _)| self.name[t])
            .collect();
        if !seen.is_empty() {
            write!(self.out, "You see {}.", seen.join(", "))?;
        }
        writeln!(self.out)
    }

    fn actually_walk_action(&mut self, targets: ThingToThing) -> io::Result<()> {
        let target = targets[self.location[Avatar]];
        if target == NoSuchThing {
            writeln!(self.out, "You can't go that way.")
        } else if self.here_before[target] {
            self.location[Avatar] = target;
            writeln!(self.out, "You are {}.", self.name[target])
        } else {
            self.location[Avatar] = target;
            self.here_before[target] = true;
            self.look_here()
        }
    }

    //******************************************************************
    // Actions

    /// The special cases that distinguish this game from others. Returns
    /// true if `c` was a special case and has been handled.
    fn special_actions(&mut self, c: &Command) -> io::Result<bool> {
        match (c.verb_id, c.object_id) {
            (DropVerb, Kettle) if self.location[Avatar] == KitchenRoom => {
                // Dropping the kettle in the kitchen places it in its stand.
                self.location[Kettle] = KitchenRoom;
                self.takeable[Kettle] = false;
                self.paragraph(concat!(
                    "The electric kettle fits snuggly into its stand &",
                    " is ready to boil water when you're ready to make",
                    " coffee!"
                ))?;
                Ok(true)
            }
            (SearchVerb, SackOfFoodstuffs) => {
                // The coffee appears wherever the sack is, and the sack,
                // no longer needed, goes nowhere.
                self.location[GroundCoffee] = self.location[SackOfFoodstuffs];
                self.location[SackOfFoodstuffs] = NoSuchThing;
                let s = format!(
                    "Searching {} reveals {}!",
                    self.name[SackOfFoodstuffs], self.name[GroundCoffee]
                );
                self.paragraph(&s)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn drink_action(&mut self, _: &Command) -> io::Result<()> {
        writeln!(self.out, "FIXME Drink_Action")
    }

    fn drop_action(&mut self, c: &Command) -> io::Result<()> {
        let s = &c.object_str;
        if c.object_id == Unsupplied {
            writeln!(self.out, "Drop what?")
        } else if c.object_id == NoSuchThing {
            writeln!(self.out, "What is a {s}?")
        } else if !self.carries(c.object_id) {
            writeln!(self.out, "You aren't carrying any {s}.")
        } else if !self.dropable[c.object_id] {
            writeln!(self.out, "It's impossible to drop {s}!")
        } else if self.special_actions(c)? {
            Ok(())
        } else {
            self.location[c.object_id] = self.location[Avatar];
            writeln!(self.out, "{s}: Dropped.")
        }
    }

    fn take_action(&mut self, c: &Command) -> io::Result<()> {
        let s = &c.object_str;
        if c.object_id == Unsupplied {
            writeln!(self.out, "Take what?")
        } else if c.object_id == NoSuchThing {
            writeln!(self.out, "What is a {s}?")
        } else if self.location[c.object_id] != self.location[Avatar] {
            writeln!(self.out, "I don't see any {s} here.")
        } else if !self.takeable[c.object_id] {
            writeln!(self.out, "You can't take that.")
        } else {
            self.location[c.object_id] = Avatar;
            writeln!(self.out, "{s}: Taken.")
        }
    }

    fn inventory_action(&mut self, _: &Command) -> io::Result<()> {
        let carried: Vec<Thing> = self
            .location
            .iter()
            .filter(|&(_, &loc)| loc == Avatar)
            .map(|(t, _)| t)
            .collect();
        write!(self.out, "You are carrying {} items.", carried.len())?;
        if carried.is_empty() {
            return writeln!(self.out);
        }
        writeln!(self.out, "  They are...")?;
        for t in carried {
            writeln!(self.out, "* {}", self.name[t])?;
        }
        Ok(())
    }

    fn look_action(&mut self, c: &Command) -> io::Result<()> {
        if !self.is_illuminated() {
            writeln!(self.out, "It's too dark to see anything.")
        } else if c.object_id == Unsupplied || c.object_id == HereRoom {
            self.look_here()
        } else if self.has_access_to(c.object_id) {
            self.paragraph(self.desc[c.object_id])
        } else {
            writeln!(self.out, "I don't see any {} here.", c.object_str)
        }
    }

    fn north_action(&mut self, _: &Command) -> io::Result<()> {
        self.actually_walk_action(self.targets_north)
    }

    fn east_action(&mut self, _: &Command) -> io::Result<()> {
        self.actually_walk_action(self.targets_east)
    }

    fn south_action(&mut self, _: &Command) -> io::Result<()> {
        self.actually_walk_action(self.targets_south)
    }

    fn west_action(&mut self, _: &Command) -> io::Result<()> {
        self.actually_walk_action(self.targets_west)
    }

    fn up_action(&mut self, _: &Command) -> io::Result<()> {
        // Unlike the other special cases, the end of the game is detected
        // after the state change.
        self.actually_walk_action(self.targets_up)?;
        if self.location[Avatar] == SunRoom {
            self.is_done = true;
        }
        Ok(())
    }

    fn down_action(&mut self, _: &Command) -> io::Result<()> {
        self.actually_walk_action(self.targets_down)
    }

    fn walk_action(&mut self, c: &Command) -> io::Result<()> {
        match c.object_id {
            NorthVerb => self.north_action(c),
            EastVerb => self.east_action(c),
            SouthVerb => self.south_action(c),
            WestVerb => self.west_action(c),
            UpVerb => self.up_action(c),
            DownVerb => self.down_action(c),
            // No newline, as in the original.
            _ => write!(self.out, "\"{}\" isn't a direction.", c.object_str),
        }
    }

    /// Has the most preconditions and the most side effects of any action.
    /// Each failed precondition tells the player what's still needed.
    fn make_action(&mut self, c: &Command) -> io::Result<()> {
        if c.object_id != GroundCoffee {
            self.paragraph(concat!(
                "You could make coffee if you have all 4 of the items &",
                " you are in the kitchen, but I'm not aware of anything",
                " else you could make in this game.  Sorry."
            ))
        } else if self.location[Avatar] != KitchenRoom {
            self.paragraph("The kitchen is the best place to make coffee.")
        } else if self.location[Kettle] != KitchenRoom {
            self.paragraph(concat!(
                "You'll need to drop an electric kettle on the electric",
                " kettle stand that's on the counter top over there."
            ))
        } else if !self.has_access_to(CleanMug) {
            self.paragraph(concat!(
                "You'll need to hold a clean mug or at least have one",
                " here in the kitchen."
            ))
        } else if !self.has_access_to(CleanFunnel) {
            self.paragraph(concat!(
                "You'll need to carry a clean pour-over funnel, or at",
                " least have one here with you in the kitchen.  And",
                " it must be clean, not just a dirty old thing."
            ))
        } else if !self.has_access_to(GroundCoffee) {
            self.paragraph(concat!(
                "You'll need to carry some ground coffee or have it",
                " here in the kitchen.  I mean, you can't make coffee",
                " without some ground coffee."
            ))
        } else {
            self.location[MugOfCoffee] = Avatar;
            self.location[CleanMug] = NoSuchThing;
            self.location[CleanFunnel] = NoSuchThing;
            self.location[GroundCoffee] = NoSuchThing;
            self.targets_up[KitchenRoom] = SunRoom;
            self.words.replace("coffee", MugOfCoffee);
            self.paragraph(concat!(
                "You fill the electric kettle with water from the faucet",
                " & start it.  You put the pour-over funnel on your mug &",
                " fill it with some ground coffee.  You wait..."
            ))?;
            self.paragraph(concat!(
                "When the water boils, you pour it into the filter &",
                " wait while it trickles into the mug."
            ))?;
            self.paragraph(concat!(
                "You now have a cup of steaming coffee!  It smells",
                " wonderful!"
            ))?;
            self.paragraph(concat!(
                "A kitty enters from some direction you didn't see,",
                " meows, & runs up some stairs you hadn't noticed",
                " before.  There's a sun room up there, perfect for",
                " sipping coffee with a friend.  Someone is waiting for",
                " you up there."
            ))
        }
    }

    /// In this game, reading something is the same as looking at it.
    fn read_action(&mut self, c: &Command) -> io::Result<()> {
        let x = c.object_id;
        if x == Unsupplied {
            writeln!(self.out, "Read what?")
        } else if x == NoSuchThing {
            writeln!(self.out, "What is a \"{}\"?", c.object_str)
        } else if self.has_access_to(x) {
            self.paragraph(self.desc[x])
        } else {
            writeln!(self.out, "I don't see any {} here.", c.object_str)
        }
    }

    /// An interesting search moves a hidden Thing from `NoSuchThing` to
    /// here, and may hide another.
    fn search_action(&mut self, c: &Command) -> io::Result<()> {
        let s = &c.object_str;
        if c.object_id == Unsupplied {
            writeln!(self.out, "Search what?")
        } else if c.object_id == NoSuchThing {
            writeln!(self.out, "What is a {s}?")
        } else if !self.has_access_to(c.object_id) {
            self.paragraph(&format!(
                "You aren't carrying any {s}, & there isn't one here in the room."
            ))
        } else if self.special_actions(c)? {
            Ok(())
        } else {
            self.paragraph(&format!("Searching {s} doesn't reveal anything."))
        }
    }

    /// Like search, swaps a Thing into hiding and another out of it.
    fn wash_action(&mut self, c: &Command) -> io::Result<()> {
        let target = c.object_id;
        if target == NoSuchThing {
            self.paragraph(&format!("What is a {}?", c.object_str))
        } else if !self.carries(target) {
            self.paragraph("You can't wash it if you aren't carrying it.")
        } else if self.location[Avatar] != KitchenRoom {
            self.paragraph(concat!(
                "The only place around here for washing anything is",
                " the kitchen."
            ))
        } else if target == DirtyMug {
            self.location[CleanMug] = self.location[DirtyMug];
            self.location[DirtyMug] = NoSuchThing;
            self.words.replace("mug", CleanMug);
            self.paragraph(concat!(
                "The formerly dirty mug is now clean enough for a drink.",
                "  Excellent!"
            ))
        } else if target == DirtyFunnel {
            self.location[CleanFunnel] = self.location[DirtyFunnel];
            self.location[DirtyFunnel] = NoSuchThing;
            self.words.replace("funnel", CleanFunnel);
            self.words.replace("filter", CleanFunnel);
            self.paragraph(concat!(
                "The pour-over funnel is now clean & ready for making",
                " coffee.  Wonderful!"
            ))
        } else {
            self.paragraph("It's cleaner than it was, always an improvement.")
        }
    }

    /// Not used in this game.
    fn wipe_action(&mut self, c: &Command) -> io::Result<()> {
        self.paragraph(&format!("FIXME You wipe {}.", c.object_str))
    }

    /// For any Thing that should never be accepted as a verb.
    fn not_a_verb_action(&mut self, c: &Command) -> io::Result<()> {
        writeln!(self.out, "\"{}\" is not a verb!", c.verb_str)
    }

    fn unsupplied_action(&mut self, _: &Command) -> io::Result<()> {
        writeln!(self.out, "Hum dee dum dum do hum.")
    }
}

/// Read one line, without its `\n`. `None` at end of input.
fn read_line(input: &mut impl BufRead) -> io::Result<Option<String>> {
    let mut buf = Vec::new();
    if input.read_until(b'\n', &mut buf)? == 0 {
        return Ok(None);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
    }
    Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
}
