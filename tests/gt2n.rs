// SPDX-FileCopyrightText: 2026 aesc silicon
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! GT2N tests.
//!
//! GT2N extends GT3, and its runset is GT3's with seven values changed and a deck for
//! the threshold-voltage layers; the fixture is the 13 nm nanosheet SVT library, every
//! cell abutted into rows, which carries one of those layers.  The other nine libraries
//! - the 31 nm sheets, and the ELVT, ULVT, HVT and LVT flavours - read the same.
//!
//! The violations the run reports are the library's own: gt2_6t_mux2_x1 lands a V0
//! flush on the end of its M1, under V0.M1.1's 4 nm, in every flavour; and on the
//! 13 nm sheets, gt2_6t_decapcc draws two VGs 5 nm from an active, under VG.ACT.1's 6.

use gdscheck::run_drc;

const LIB: &str = "tests/data/gt2n/static/gt2_6t_std_cells_w13_svt.gds.gz";

#[test]
fn the_standard_cell_library_holds_only_its_own_violations() {
    let violations =
        run_drc(LIB, "gt2n", &[], Some("main"), "ALLCELLS", true).expect("DRC run failed");
    let mut ids: Vec<&str> = violations.iter().map(|v| v.rule_id.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(ids, ["V0.M1.1", "VG.ACT.1", "VG.ACT.1"]);
}
