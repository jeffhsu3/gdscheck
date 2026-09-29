// SPDX-FileCopyrightText: 2026 aesc silicon
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ASAP7 tests.
//!
//! The Calibre deck the manual was written against is not public, so there are no
//! foundry test cases to run the decks on.  What there is, is the standard-cell
//! library the PDK ships, which was verified with that deck: every cell of it, placed
//! once, is the fixture.  A rule that reads the manual wrongly shows up here as a
//! violation on a cell that has none.
//!
//! The few violations the run does report are the library's own.  The half-width
//! filler FILLERxp5 and the TAPCELL_WITH_FILLER are 54 nm wide, under the 108 nm the
//! manual asks of a well, an implant and a fin across, which only abutting cells make
//! up; and DFFASRHQNx1 draws one LIG 3 nm from an SDT.  The KLayout port of the manual
//! reports these too.  FILLERxp5's gate also has no neighbour until a cell is placed
//! beside it, which the port's GATE.S.3 - a gate grown and tested against itself -
//! cannot see.

use gdscheck::run_drc;

const PDK: &str = "asap7";
const LIB: &str = "tests/data/asap7/static/asap7sc7p5t_28_L.gds.gz";

fn counts(gds: &str, suite: &str) -> Vec<(String, usize)> {
    let violations = run_drc(gds, PDK, &[], Some(suite), "ALLCELLS", true).expect("DRC run failed");
    let mut by_rule: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for v in violations {
        *by_rule.entry(v.rule_id).or_default() += 1;
    }
    by_rule.into_iter().collect()
}

/// The whole library under the full run: only the violations it is known to hold.  A
/// width is two markers, one per wall.
#[test]
fn the_standard_cell_library_holds_only_its_own_violations() {
    let want: Vec<(String, usize)> = [
        ("FIN.W.2", 20),
        ("GATE.S.3", 2),
        ("LIG.SDT.S.8", 1),
        ("LVT.W.1", 2),
        ("NSELECT.W.1", 4),
        ("PSELECT.W.1", 4),
        ("WELL.W.1", 2),
    ]
    .iter()
    .map(|(id, n)| ((*id).to_string(), *n))
    .collect();
    assert_eq!(counts(LIB, "main"), want);
}

/// The front-end and back-end suites split the full run between them: every deck is
/// in one or the other, and none in both.
#[test]
fn the_feol_and_beol_suites_split_main() {
    let pdk = gdscheck::pdk::PdkConfig::for_process(PDK).expect("PDK loads");
    let ids = |suite: &str| -> std::collections::BTreeSet<String> {
        pdk.load_suite(suite)
            .expect("suite loads")
            .iter()
            .map(|r| {
                let layers: Vec<&str> = r.layers.iter().map(|l| l.name.as_str()).collect();
                format!("{} {} {:?} {}", r.id, r.check, layers, r.value)
            })
            .collect()
    };
    let (main, feol, beol) = (ids("main"), ids("feol"), ids("beol"));
    assert!(feol.is_disjoint(&beol), "a rule in both feol and beol");
    let both: std::collections::BTreeSet<String> = feol.union(&beol).cloned().collect();
    let missing: Vec<&String> = main.difference(&both).collect();
    assert!(missing.is_empty(), "in main, in neither half: {missing:?}");
}
