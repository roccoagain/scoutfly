//! How much of each test board the built-in library covers, so a change that loses coverage fails here.
use std::path::PathBuf;

use gerberus::kicad;
use gerberus::limits::{Coverage, Library, coverage};

fn root(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// How many of each test board's fitted parts a built-in part file or family covers. Raise a number when a new
/// part file covers more; it may never fall.
#[test]
fn the_built_in_library_covers_the_test_boards() {
    let lib = Library::builtin().unwrap();
    for (board, covered) in [
        ("can-encoder/can-encoder.kicad_sch", 30),
        ("can-encoder-rev1/can-encoder.kicad_sch", 28),
        ("kitchen-sink/kitchen-sink.kicad_sch", 71),
        ("sniffer/sniffer.kicad_sch", 15),
    ] {
        let design = kicad::load(&root("tests/fixtures").join(board)).unwrap();
        let cov = coverage(&design, &lib);
        let by_file = cov.values().filter(|c| **c == Coverage::Datasheet).count();
        assert!(by_file >= covered, "{board}: {by_file} covered, down from {covered}");
    }
}
