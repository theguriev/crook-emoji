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
    assert_eq!(mark_for(&row(key)), MARKS[(mixed(key) % 64) as usize]);
}

/// The key the host gives the `nth` tab working in `directory`, the way
/// `crook`'s `plugins::wasm::keyed` makes it: FNV-1a over the plugin's id,
/// a separator, the directory, and — past the first tab there — a separator
/// and the tab's place among them.
fn host_key(directory: &str, nth: u64) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let fold = |mut hash: u64, bytes: &[u8]| {
        for &byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(PRIME);
        }
        hash
    };
    let key = fold(
        fold(fold(OFFSET, b"theguriev/emoji"), &[0]),
        directory.as_bytes(),
    );
    match nth {
        0 => key,
        nth => fold(fold(key, &[0]), &nth.to_le_bytes()),
    }
}

#[test]
fn two_tabs_in_one_directory_share_a_mark_about_as_often_as_chance() {
    // Every tab starts where Crook was started, so tabs in one place are the
    // ordinary case, and the key is all that tells them apart. The second and
    // the tenth there wore the same mark in 318 directories of 1024; chance
    // is sixteen.
    let directories = 1024;
    for (first, second) in [(1, 9), (2, 10), (1, 4)] {
        let same = (0..directories)
            .filter(|index| {
                let directory = format!("/home/someone/Work/project-{index}");
                mark_for(&row(host_key(&directory, first)))
                    == mark_for(&row(host_key(&directory, second)))
            })
            .count();
        assert!(
            same <= 48,
            "tabs {first} and {second} share a mark in {same} of {directories} directories"
        );
    }
}

#[test]
fn a_dozen_tabs_in_one_directory_wear_about_as_many_marks_as_chance_gives() {
    // Twelve picks from sixty-four have eleven different ones on average; the
    // unmixed key gave fewer than ten.
    let directories = 256;
    let distinct: usize = (0..directories)
        .map(|index| {
            let directory = format!("/srv/checkouts/repo-{index}");
            let mut marks: Vec<&str> = (0..12)
                .map(|nth| mark_for(&row(host_key(&directory, nth))))
                .collect();
            marks.sort_unstable();
            marks.dedup();
            marks.len()
        })
        .sum();
    let average = distinct as f64 / f64::from(directories);
    assert!(
        average >= 10.5,
        "a dozen tabs wear {average:.2} different marks"
    );
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
