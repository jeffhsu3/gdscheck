// SPDX-FileCopyrightText: 2026 aesc silicon
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generated GT2N rule patterns, in nm on the 0.5 nm grid of GT2N's libraries.
//!
//! Each family writes good/bad GDS fixtures under `tests/data/gt2n/generated/<deck>/`.
//! Expectations live in `tests/gt2n.rs`. Limits come from GT2N's IC Validator runset
//! (`icv_runset/Include/gt2.*.drc.rs`); the generator checks the decks against them,
//! so a value changed in a deck - GT2N's own or one it shares with GT3 - stops it.

mod beol;
mod feol;
mod mol;

use crate::helpers::{library, poly, rect, write_gz};
use gds21::{GdsElement, GdsUnits};
use gdscheck::pdk::{PdkConfig, RuleDefinition};

const DIR: &str = "tests/data/gt2n/generated";
const DBU: f64 = 0.5;

/// `v` nm in the units of the shared helpers, which draw in µm on a 1 nm DBU. GT2N
/// draws on a 0.5 nm DBU, so each helper nanometre stands for one GT2N DBU here, and
/// [`Corpus::write`] stamps the 0.5 nm units on the result.
fn nm(v: f64) -> f64 {
    v * 2.0 / 1000.0
}

/// The smallest `dy` on the grid that puts the corners of two shapes `dx` apart in x
/// at least `limit` apart, the pair for a corner-to-corner rule: `dy` reads clean and
/// `dy` less a DBU does not. The grid holds 15 and 45 nm diagonals exactly (9 by 12, 27
/// by 36); for 16, 18, 24 and 27 nm the clean corner is the nearest one past the limit.
fn corner_rise(limit: f64, dx: f64) -> f64 {
    let mut dy = DBU;
    while dx * dx + dy * dy < limit * limit {
        dy += DBU;
    }
    assert!(dx * dx + (dy - DBU) * (dy - DBU) < limit * limit);
    dy
}

/// Room between the rows of a pattern: past every spacing any of its shapes answers
/// to, but BPR.3's 10 µm along a rail.
const ROOM: f64 = 500.0;

/// A `w`×`h` box with its lower-left corner at (x, y), in nm.
fn bx(layer: (i16, i16), x: f64, y: f64, w: f64, h: f64) -> GdsElement {
    rect(layer, nm(x), nm(y), nm(x + w), nm(y + h))
}

/// A polygon through `pts`, in nm, closed for you.
fn pg(layer: (i16, i16), pts: &[(f64, f64)]) -> GdsElement {
    let pts: Vec<_> = pts.iter().map(|&(x, y)| (nm(x), nm(y))).collect();
    poly(layer, &pts)
}

/// Rows of a pattern, stacked upward from `y` with [`ROOM`] between them.
struct Rows {
    y: f64,
}

impl Rows {
    fn new(y: f64) -> Self {
        Rows { y }
    }

    /// The bottom of a row `h` tall, the next row starting above it.
    fn next(&mut self, h: f64) -> f64 {
        let y = self.y;
        self.y += h + ROOM;
        y
    }
}

struct Corpus<'a> {
    pdk: &'a PdkConfig,
    rules: Vec<RuleDefinition>,
}

impl Corpus<'_> {
    /// The runset's `nm` for rule `id`, after checking every `check` entry of it in the
    /// deck says the same.
    fn src(&self, id: &str, check: &str, nm: f64) -> f64 {
        let found: Vec<_> = self
            .rules
            .iter()
            .filter(|r| r.id == id && r.check == check)
            .collect();
        assert!(!found.is_empty(), "{id} has no {check} entry in any deck");
        for r in found {
            assert!(
                (r.value * 1000.0 - nm).abs() < 1e-6,
                "{id} {check}: deck value is {} nm, the runset says {nm} nm",
                r.value * 1000.0
            );
        }
        nm
    }

    /// As [`Self::src`], for the numeric param `key`.
    fn src_param(&self, id: &str, check: &str, key: &str, nm: f64) -> f64 {
        let found: Vec<_> = self
            .rules
            .iter()
            .filter(|r| r.id == id && r.check == check)
            .collect();
        assert!(!found.is_empty(), "{id} has no {check} entry in any deck");
        for r in found {
            let um = r
                .num(key)
                .unwrap_or_else(|| panic!("{id} {check} has no numeric {key}"));
            assert!(
                (um * 1000.0 - nm).abs() < 1e-6,
                "{id} {check}: deck {key} is {} nm, the runset says {nm} nm",
                um * 1000.0
            );
        }
        nm
    }

    /// The radius in nm of the sized virtual layer `name`, one step of a sentence.
    fn radius(&self, name: &str) -> f64 {
        let def = self
            .pdk
            .virtual_layers
            .iter()
            .find(|v| v.name == name)
            .unwrap_or_else(|| panic!("no virtual layer {name}"));
        def.radius.unwrap_or_else(|| panic!("{name} has no radius")) * 1000.0
    }

    fn layer(&self, name: &str) -> (i16, i16) {
        crate::helpers::layer(self.pdk, name)
    }

    /// Write `elems`, drawn in [`nm`] units, as `<deck>/<name>.<good|bad>.gds.gz`.
    fn write(&self, deck: &str, name: &str, bad: bool, elems: Vec<GdsElement>) {
        let mut lib = library("TOP", elems);
        lib.units = GdsUnits(0.0005, 0.5e-9);
        let dir = format!("{DIR}/{deck}");
        std::fs::create_dir_all(&dir).expect("pattern directory");
        let polarity = if bad { "bad" } else { "good" };
        write_gz(&format!("{dir}/{name}.{polarity}.gds.gz"), lib);
    }
}

pub fn generate(pdk: &PdkConfig) {
    let rules = pdk
        .decks
        .iter()
        .flat_map(|d| {
            pdk.load_deck(&d.name)
                .unwrap_or_else(|e| panic!("failed to load deck '{}': {e}", d.name))
        })
        .collect();
    let c = Corpus { pdk, rules };
    feol::generate(&c);
    mol::generate(&c);
    beol::generate(&c);
}
