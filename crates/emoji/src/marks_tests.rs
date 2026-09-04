//! What the table promises.

use super::*;

/// A row with nothing in it but its key, which is every row this plugin sees:
/// it asks for no capability, so the host redacts the rest.
fn row(key: u64) -> TabFacts {
    TabFacts {
        key,
        tab: None,
        place: None,
    }
}

#[test]
fn the_same_tab_gets_the_same_mark_every_time() {
    // The whole promise. A mark that changed between two frames would be a
    // panel that flickers; one that changed between two runs would be a mark
    // nobody could learn.
    let key = 0x1234_5678_9abc_def0;

    assert_eq!(mark_for(&row(key)), mark_for(&row(key)));
    assert_eq!(
        mark_for(&row(key)),
        MARKS[(key ^ (key >> 32)) as usize % 64]
    );
}

#[test]
fn neighbouring_keys_do_not_get_neighbouring_marks() {
    // Not a property of this file — it is the host's hash that has it — but
    // the one worth asserting from here, because a version of this that
    // indexed by tab position instead would pass every other test in the file
    // and give the whole panel five fruit in a row.
    let marks: Vec<&str> = (0..8)
        .map(|index| mark_for(&row(0xcbf2_9ce4_8422_2325u64.wrapping_mul(index + 1))))
        .collect();
    let mut unique = marks.clone();
    unique.sort_unstable();
    unique.dedup();

    assert_eq!(unique.len(), marks.len(), "{marks:?}");
}

#[test]
fn every_mark_is_one_code_point_and_none_needs_a_variation_selector() {
    // The rule the table is chosen by, asserted rather than trusted: a mark
    // that is two scalars can be drawn as two glyphs by a shaper that does not
    // join them, and a mark below U+1F000 can be drawn as grey text.
    for mark in MARKS {
        let mut chars = mark.chars();
        let first = chars.next().expect("a mark is not empty");

        assert_eq!(chars.next(), None, "{mark:?} is more than one code point");
        assert!(first as u32 >= 0x1_f000, "{mark:?} may need a U+FE0F");
    }
}

#[test]
fn no_mark_is_in_the_table_twice() {
    let mut unique = MARKS.to_vec();
    unique.sort_unstable();
    unique.dedup();

    assert_eq!(unique.len(), MARKS.len(), "a mark is in the table twice");
}
