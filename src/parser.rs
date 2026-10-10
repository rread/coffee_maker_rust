//! The 2-word parser: split a line into words, then bind each word to a
//! Thing through the dictionary.

use crate::tables::WORDS;
use crate::thing::Thing;

/// A parsed command. Each word is kept along with the Thing it's bound to;
/// a word the player didn't type is `""` and binds to `Unsupplied`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Command {
    pub verb_str: String,
    pub verb_id: Thing,
    pub object_str: String,
    pub object_id: Thing,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TooManyWords;

/// Split a line into at most two words separated by `' '`. A missing word
/// is `""`.
pub fn split_into_words(line: &str) -> Result<(&str, &str), TooManyWords> {
    let mut words = line.split(' ').filter(|w| !w.is_empty());
    let verb = words.next().unwrap_or("");
    let object = words.next().unwrap_or("");
    match words.next() {
        Some(_) => Err(TooManyWords),
        None => Ok((verb, object)),
    }
}

/// Maps words to Things. Searched linearly, like the original.
pub struct Words(Vec<(&'static str, Thing)>);

impl Words {
    pub fn new() -> Self {
        Words(WORDS.to_vec())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn search(&self, word: &str) -> Option<Thing> {
        self.0.iter().find(|(w, _)| *w == word).map(|&(_, t)| t)
    }

    /// Direct a word in the dictionary to a different Thing.
    pub fn replace(&mut self, word: &str, replacement: Thing) {
        let entry = self
            .0
            .iter_mut()
            .find(|(w, _)| *w == word)
            .unwrap_or_else(|| panic!("replace: {word:?} is not in the dictionary"));
        entry.1 = replacement;
    }

    pub fn parse(&self, line: &str) -> Result<Command, TooManyWords> {
        let (verb, object) = split_into_words(line)?;
        Ok(Command {
            verb_str: verb.to_owned(),
            verb_id: self.search(verb).unwrap_or(Thing::NoSuchVerb),
            object_str: object.to_owned(),
            object_id: self.search(object).unwrap_or(Thing::NoSuchThing),
        })
    }
}

impl Default for Words {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_examples_from_ada_source() {
        assert_eq!(split_into_words(""), Ok(("", "")));
        assert_eq!(split_into_words("a"), Ok(("a", "")));
        assert_eq!(split_into_words(" a "), Ok(("a", "")));
        assert_eq!(split_into_words("      "), Ok(("", "")));
        assert_eq!(split_into_words(" word1   word2 "), Ok(("word1", "word2")));
        assert_eq!(split_into_words("one two three"), Err(TooManyWords));
        assert_eq!(split_into_words("look\tnote"), Ok(("look\tnote", "")));
    }

    #[test]
    fn parse_binds_words() {
        let words = Words::new();
        let c = words.parse("look xyzzy").unwrap();
        assert_eq!(
            (c.verb_id, c.object_id),
            (Thing::LookVerb, Thing::NoSuchThing)
        );
        let c = words.parse("frob").unwrap();
        assert_eq!(
            (c.verb_id, c.object_id),
            (Thing::NoSuchVerb, Thing::Unsupplied)
        );
        let c = words.parse("").unwrap();
        assert_eq!(c.verb_id, Thing::Unsupplied);
    }

    #[test]
    fn replace() {
        let mut words = Words::new();
        assert_eq!(words.len(), 55);
        words.replace("mug", Thing::CleanMug);
        assert_eq!(words.search("mug"), Some(Thing::CleanMug));
    }
}
