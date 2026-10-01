// SPDX-FileCopyrightText: 2026 aesc silicon
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! GT3 tests.
//!
//! The decks translate GT3's IC Validator runset, which needs a licence to run, so the
//! fixture is the standard-cell library the PDK ships: every cell, abutted into rows
//! on its placement boundary the way a placer would.  A rule translated wrongly shows
//! up as a violation on a cell that has none.
//!
//! The one violation the run reports is the library's own: placed beside
//! gt3_6t_oa211_x1_rvt, an M0 of gt3_6t_nor3_x2_rvt ends 16.5 nm from the neighbour's,
//! under M0.4's 17 nm tip to tip.

use gdscheck::run_drc;

const LIB: &str = "tests/data/gt3/static/gt3_6t_std_cell_rvt.gds.gz";

#[test]
fn the_standard_cell_library_holds_only_its_own_violations() {
    let violations =
        run_drc(LIB, "gt3", &[], Some("main"), "ALLCELLS", true).expect("DRC run failed");
    let ids: Vec<&str> = violations.iter().map(|v| v.rule_id.as_str()).collect();
    assert_eq!(ids, ["M0.4"]);
}
