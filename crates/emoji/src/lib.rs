//! An emoji at the head of every tab, in place of the status dot.
//!
//! Crook's tab panel draws a small coloured disc at the front of every row.
//! This replaces it with a picture: the same picture for the same tab, every
//! frame and every day, and a different one for the tab under it. That is the
//! whole plugin.
//!
//! # What it is allowed to do, which is nothing
//!
//! Its manifest asks for no capability at all, and this is the interesting
//! part rather than a footnote. A mark per tab sounds like something that has
//! to *read* the tabs — their names, their directories, what each agent is
//! doing — and it does not. The host hands a contribution to a row slot one
//! number per row: the same number for one tab every time, a different one for
//! the tab beside it, and nothing recoverable about either. A remainder over a
//! table turns that into a picture.
//!
//! So this plugin cannot tell you what any of your tabs are called, and no
//! permission dialog stands between installing it and seeing it work.
//!
//! # What this file is
//!
//! The ABI, and nothing that thinks. Every export the host calls is here, each
//! of them is three lines, and each hands over to [`marks`] — which is a plain
//! Rust module `cargo test` runs on an ordinary machine. See [`sys`] for the
//! other half of that trick.

use std::cell::UnsafeCell;

use crook_plugin_api::{
    ABI_VERSION, Manifest, Node, Render, Size, Subject, Tone, from_bytes, to_bytes,
};

pub mod marks;
pub mod sys;

use marks::mark_for;

/// The slot this plugin takes: the 24px mark at the head of a tab's row.
const SLOT: &str = "tab.row.mark";

/// What this contribution is called, within this plugin.
const ENTRY: &str = "mark";

/// Where it goes among everything else in that slot.
///
/// The slot holds one thing and the lowest order wins it, so this is a claim
/// rather than a position: a person who installs this plugin wants the emoji
/// and not the dot. Zero rather than a large negative number, so that a plugin
/// somebody installs *later* and means more by can still take the slot back.
const ORDER: i32 = 0;

/// A `static` that is only ever touched by one thread, which on wasm32 is
/// every thread there is.
struct Single<T>(UnsafeCell<T>);

// SAFETY: wasm32 has one thread. Off wasm this crate is a library under test,
// where nothing reaches this static at all.
unsafe impl<T> Sync for Single<T> {}

impl<T> Single<T> {
    /// SAFETY: the caller must not be inside another borrow. Every export
    /// below takes one, does its work and returns, and the host does not call
    /// in while it is already inside.
    #[allow(clippy::mut_from_ref)]
    unsafe fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

/// The last thing handed back to the host, kept alive until the next one.
///
/// A tree is answered as an offset and a length into this memory, so the bytes
/// have to outlive the call that returned them. Kept rather than leaked
/// because a render happens once per row per frame, and a leak per row per
/// frame is a plugin that eventually stops fitting in its own memory.
static ANSWER: Single<Vec<u8>> = Single(UnsafeCell::new(Vec::new()));

/// Packs an answer as the host reads it: `(pointer << 32) | length`.
fn hand_back(bytes: Vec<u8>) -> i64 {
    // SAFETY: see `Single::get`.
    let answer = unsafe { ANSWER.get() };
    *answer = bytes;
    ((answer.as_ptr() as u64) << 32 | answer.len() as u64) as i64
}

/// Which version of the vocabulary this was built against.
///
/// Called before anything else, and a mismatch is a refusal by number rather
/// than a plugin that decodes a shape which means something else now.
#[unsafe(no_mangle)]
pub extern "C" fn crook_abi_version() -> i32 {
    ABI_VERSION as i32
}

/// Somewhere for the host to put the bytes it is handing over.
///
/// Exact rather than `Vec::with_capacity`, because [`take`] frees it with the
/// same layout and a capacity the allocator rounded up would be a free of the
/// wrong size.
#[unsafe(no_mangle)]
pub extern "C" fn crook_alloc(length: i32) -> i32 {
    let Ok(layout) = std::alloc::Layout::from_size_align(length.max(1) as usize, 1) else {
        return 0;
    };
    // SAFETY: a non-zero size, and a layout built for it.
    unsafe { std::alloc::alloc(layout) as i32 }
}

/// Copies out what the host wrote there, and gives the memory back.
///
/// SAFETY: `pointer` and `length` must be exactly what a previous
/// [`crook_alloc`] answered and what the host wrote into.
unsafe fn take(pointer: i32, length: i32) -> Vec<u8> {
    if pointer <= 0 || length < 0 {
        return Vec::new();
    }
    // SAFETY: the host wrote `length` bytes at `pointer` before calling in.
    let bytes =
        unsafe { std::slice::from_raw_parts(pointer as *const u8, length as usize) }.to_vec();
    // SAFETY: the same layout `crook_alloc` used.
    unsafe {
        std::alloc::dealloc(
            pointer as *mut u8,
            std::alloc::Layout::from_size_align_unchecked(length.max(1) as usize, 1),
        );
    }
    bytes
}

/// What this plugin is and what it needs to be allowed to do.
///
/// Read before any of it runs, which is what lets a person see what it wants
/// and refuse it without running a line of it. There is nothing here to
/// refuse.
#[unsafe(no_mangle)]
pub extern "C" fn crook_manifest() -> i64 {
    hand_back(to_bytes(&manifest()).unwrap_or_default())
}

/// The manifest, as a value, so that a test can read it.
pub fn manifest() -> Manifest {
    Manifest {
        abi: ABI_VERSION,
        id: String::from("theguriev/emoji"),
        name: String::from("Emoji"),
        description: String::from("An emoji at the head of every tab, instead of the status dot."),
        version: String::from(env!("CARGO_PKG_VERSION")),
        capabilities: Vec::new(),
    }
}

/// Takes the mark at the head of a row, and registers nothing else.
///
/// No action, because there is nothing to press; no timer, because there is
/// nothing to wait for; no request, because there is nothing to ask. A plugin
/// whose build is one call is a plugin whose whole behaviour is on the next
/// screen of this file.
#[unsafe(no_mangle)]
pub extern "C" fn crook_build() -> i32 {
    sys::contribute(SLOT, ENTRY, ORDER);
    0
}

/// What to draw on one row.
#[unsafe(no_mangle)]
pub extern "C" fn crook_render(pointer: i32, length: i32) -> i64 {
    // SAFETY: the host allocated and wrote this before calling in.
    let bytes = unsafe { take(pointer, length) };
    hand_back(to_bytes(&tree(from_bytes::<Render>(&bytes).ok())).unwrap_or_default())
}

/// What one render comes to.
///
/// A function over the request rather than a body inside the export, so that
/// the answer to "what does it draw when the host asks about something it does
/// not know" is a thing a test can ask.
///
/// Every unexpected shape draws **nothing** rather than a guess: a request
/// this build cannot decode, a slot this plugin did not contribute to, a
/// subject that is not a tab. Drawing nothing in a row slot is not a hole —
/// the host puts back whatever it would have drawn without a plugin, which
/// here is the status dot.
pub fn tree(render: Option<Render>) -> Node {
    let Some(render) = render else {
        return Node::Empty;
    };
    let (true, Some(Subject::Tab(facts))) = (render.slot == SLOT, render.subject) else {
        return Node::Empty;
    };

    Node::Text {
        text: String::from(mark_for(&facts)),
        // The size that fills the box the dot was drawn in. What that comes to
        // in pixels is the host's business and changes with the display; this
        // says which of the three sizes it is and nothing more.
        size: Size::Body,
        // An emoji is a colour glyph and carries its own, so the tone reaches
        // nothing here. Primary is what it *would* be if this build drew the
        // mark in text, which is the honest answer rather than a tone chosen
        // because it looked right on one machine.
        tone: Tone::Primary,
    }
}

/// Runs one of the actions registered while building, of which there are none.
///
/// Exported anyway. The host is entitled to call it, and a module missing an
/// export the ABI names is a plugin that fails at the boundary rather than one
/// that politely does nothing.
#[unsafe(no_mangle)]
pub extern "C" fn crook_run(pointer: i32, length: i32) -> i32 {
    // SAFETY: as above. Taken and dropped, so the host's allocation is freed
    // rather than leaked once per call.
    let name = unsafe { take(pointer, length) };
    sys::log(
        sys::Level::Warn,
        &format!(
            "asked to run {:?}, which this plugin never registered",
            String::from_utf8_lossy(&name)
        ),
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crook_plugin_api::TabFacts;

    /// A request about one row, which is all this plugin is ever sent.
    fn about(key: u64) -> Render {
        Render {
            slot: String::from(SLOT),
            entry: String::from(ENTRY),
            subject: Some(Subject::Tab(TabFacts {
                key,
                tab: None,
                place: None,
            })),
        }
    }

    #[test]
    fn it_asks_to_be_allowed_nothing() {
        // The claim the README makes, and the one thing about this plugin
        // worth protecting: a mark per tab needs no permission, so a version
        // of it that quietly started wanting one should fail here first.
        assert!(manifest().capabilities.is_empty());
        assert_eq!(manifest().abi, ABI_VERSION);
        assert_eq!(manifest().id, "theguriev/emoji");
    }

    #[test]
    fn a_row_is_drawn_as_its_own_mark() {
        let Node::Text { text, .. } = tree(Some(about(7))) else {
            panic!("a row should be drawn as text");
        };

        assert_eq!(
            text,
            mark_for(&TabFacts {
                key: 7,
                tab: None,
                place: None
            })
        );
    }

    #[test]
    fn anything_it_was_not_asked_draws_nothing() {
        // A row slot's "nothing" is not a hole: the host draws the dot it
        // would have drawn anyway. So an unknown slot, an unknown subject and
        // an undecodable request all end in the panel looking untouched rather
        // than looking broken.
        assert_eq!(tree(None), Node::Empty);
        assert_eq!(
            tree(Some(Render {
                slot: String::from("header.right"),
                entry: String::from(ENTRY),
                subject: None,
            })),
            Node::Empty
        );
        assert_eq!(
            tree(Some(Render {
                slot: String::from(SLOT),
                entry: String::from(ENTRY),
                subject: None,
            })),
            Node::Empty
        );
    }

    #[test]
    fn the_manifest_crosses_the_wire_as_itself() {
        let bytes = to_bytes(&manifest()).expect("a manifest should encode");

        assert_eq!(
            from_bytes::<Manifest>(&bytes).expect("and decode"),
            manifest()
        );
    }
}
