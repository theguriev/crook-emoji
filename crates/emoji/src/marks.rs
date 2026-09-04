//! Which emoji a tab gets, and why it is always the same one.
//!
//! The whole of this plugin's thinking, and it is a table and a remainder.
//! Kept away from [`crate::sys`] so that `cargo test` runs it on an ordinary
//! machine: what a plugin *decides* should not need a terminal to install it
//! into before anybody can find out whether it is right.

use crook_plugin_api::TabFacts;

/// The marks a tab can be given.
///
/// Three rules decided what is in here, and each of them removed more
/// candidates than it kept.
///
/// **One code point each.** No flags, no keycaps, no skin tones, no
/// zero-width joins — a family is seven code points and a shaper that fails to
/// join them draws all seven. Every mark below is a single scalar in the
/// supplementary plane, which is the one shape that cannot come apart.
///
/// **No variation selectors.** `☂` and `⭐` are older characters that need a
/// `U+FE0F` after them to be drawn as pictures rather than as text, and a
/// terminal that draws the text form puts a grey outline where a mark should
/// be. Nothing here is from the plane where that question arises.
///
/// **Different at sixteen pixels.** The mark is drawn where a status disc was,
/// which is a little under twenty pixels across. Two emoji that are both a
/// small round fruit are one emoji at that size, so the table is spread across
/// creatures, plants, food, weather and objects rather than filled up from any
/// one of them.
///
/// Sixty-four of them, which is not a coincidence: it is enough that a person
/// with a dozen tabs open rarely sees two the same, and few enough that every
/// one of them could be looked at and kept on purpose.
pub const MARKS: [&str; 64] = [
    "\u{1f419}", // octopus
    "\u{1f98a}", // fox
    "\u{1f422}", // turtle
    "\u{1f989}", // owl
    "\u{1f41d}", // honeybee
    "\u{1f98b}", // butterfly
    "\u{1f42c}", // dolphin
    "\u{1f994}", // hedgehog
    "\u{1f428}", // koala
    "\u{1f438}", // frog
    "\u{1f9a9}", // flamingo
    "\u{1f9a5}", // sloth
    "\u{1f433}", // whale
    "\u{1f996}", // t-rex
    "\u{1f427}", // penguin
    "\u{1f984}", // unicorn
    "\u{1f40c}", // snail
    "\u{1f99c}", // parrot
    "\u{1f421}", // blowfish
    "\u{1f43f}", // chipmunk
    "\u{1f34e}", // red apple
    "\u{1f34a}", // tangerine
    "\u{1f34b}", // lemon
    "\u{1f347}", // grapes
    "\u{1f349}", // watermelon
    "\u{1f351}", // peach
    "\u{1f352}", // cherries
    "\u{1f95d}", // kiwi
    "\u{1f951}", // avocado
    "\u{1f344}", // mushroom
    "\u{1f966}", // broccoli
    "\u{1f95e}", // pancakes
    "\u{1f36d}", // lollipop
    "\u{1f9c1}", // cupcake
    "\u{1f345}", // tomato
    "\u{1f955}", // carrot
    "\u{1f680}", // rocket
    "\u{1f6f8}", // flying saucer
    "\u{1f388}", // balloon
    "\u{1f3a8}", // artist palette
    "\u{1f3af}", // bullseye
    "\u{1f3b8}", // guitar
    "\u{1f3ba}", // trumpet
    "\u{1f52e}", // crystal ball
    "\u{1f9ed}", // compass
    "\u{1f52d}", // telescope
    "\u{1f9f2}", // magnet
    "\u{1fa81}", // kite
    "\u{1f6fc}", // roller skate
    "\u{1f9ff}", // nazar amulet
    "\u{1f9e9}", // puzzle piece
    "\u{1f3b2}", // game die
    "\u{1f335}", // cactus
    "\u{1f33b}", // sunflower
    "\u{1f338}", // cherry blossom
    "\u{1f341}", // maple leaf
    "\u{1f30a}", // wave
    "\u{1f525}", // fire
    "\u{1f308}", // rainbow
    "\u{1f319}", // crescent moon
    "\u{1f30b}", // volcano
    "\u{1f9ca}", // ice
    "\u{1f4a1}", // light bulb
    "\u{1f9f6}", // yarn
];

/// The mark for one row.
///
/// The key is the host's, and it is the reason this plugin needs to be allowed
/// nothing at all: it says which row this is and nothing else about it — the
/// same number every time for the same tab, a different one for the tab beside
/// it, and the same one again the next time that project is opened. So the
/// choice is a remainder, and a person who liked the octopus their `crook`
/// checkout got keeps it.
///
/// The high bits are folded in first. The host's key is a hash and its low
/// bits are as good as its high ones today, but a remainder that reads only
/// the bottom six is a plugin that would start repeating itself if that ever
/// stopped being true — and folding is two instructions.
pub fn mark_for(facts: &TabFacts) -> &'static str {
    let key = facts.key ^ (facts.key >> 32);
    MARKS[(key % MARKS.len() as u64) as usize]
}

#[cfg(test)]
#[path = "marks_tests.rs"]
mod tests;
