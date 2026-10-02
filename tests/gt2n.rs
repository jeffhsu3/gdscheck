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

use gds21::{GdsBoundary, GdsElement, GdsLibrary, GdsPoint, GdsStruct, GdsUnits};
use gdscheck::run_drc;
use gdscheck::{connectivity::Connectivity, flatten, merge, parse_virtual_op, pdk::PdkConfig};
use rstest::rstest;
use std::collections::HashMap;

const LIB: &str = "tests/data/gt2n/static/gt2_6t_std_cells_w13_svt.gds.gz";

#[test]
fn the_standard_cell_library_holds_only_its_own_violations() {
    let violations =
        run_drc(LIB, "gt2n", &[], Some("main"), "ALLCELLS", true).expect("DRC run failed");
    let mut ids: Vec<&str> = violations.iter().map(|v| v.rule_id.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(ids, ["V0.M1.1", "VG.ACT.1", "VG.ACT.1"]);
}

// Physical GDS layer numbers from GT2N's gt2_main.drc.rs. Drawing the source numbers
// independently of PdkConfig also catches a wrong layer mapping in the port.
const BPR: i16 = 8;
const VBPR: i16 = 9;
const SDCON: i16 = 10;
const BACK_METALS: [i16; 4] = [920, 940, 960, 980];
const BACK_VIAS: [i16; 5] = [910, 930, 950, 970, 990];
const BRDL: i16 = 1000;

fn rect(layer: i16, [x0, y0, x1, y1]: [i32; 4]) -> GdsElement {
    GdsElement::GdsBoundary(GdsBoundary {
        layer,
        datatype: 0,
        xy: [(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)]
            .map(|(x, y)| GdsPoint::new(x * 2, y * 2))
            .to_vec(),
        ..Default::default()
    })
}

fn library(elems: Vec<GdsElement>) -> GdsLibrary {
    let mut top = GdsStruct::new("TOP");
    top.elems = elems;
    let mut lib = GdsLibrary::new("BACKSIDE");
    lib.units = GdsUnits(0.0005, 0.5e-9);
    lib.structs = vec![top];
    lib
}

fn extract(lib: &GdsLibrary, tile_um: f64) -> Connectivity {
    let pdk = PdkConfig::for_process("gt2n").unwrap();
    let dbu_to_um = lib.units.1 * 1e6;
    let mut layout = flatten::flatten_to_elems("TOP", lib, None, &[]);
    let eager = pdk.eager_layers(&[]).unwrap();
    pdk.compute_virtual_layers(&mut layout, dbu_to_um, &eager);
    let mut cache = merge::MergedCache::new(
        (tile_um / dbu_to_um).round() as i32,
        (merge::MIN_HALO_UM / dbu_to_um).ceil() as i32,
        HashMap::new(),
    );
    for spec in pdk.tiled_virtual_layers(&eager) {
        let op = parse_virtual_op(
            &spec.op,
            spec.radius,
            spec.min,
            spec.max,
            spec.slack,
            dbu_to_um,
        )
        .unwrap();
        cache.register_virtual(spec.key, op, spec.sources, spec.text);
    }
    Connectivity::build(&mut cache, &layout, &pdk.connectivity, dbu_to_um, &[])
}

fn net(conn: &Connectivity, layer: i16, x_nm: i32, y_nm: i32) -> usize {
    conn.net_at((layer, 0), f64::from(x_nm * 2), f64::from(y_nm * 2))
        .expect("conductor must have a net even when disconnected")
}

/// Two 16 nm SDCONs, 19 nm apart: SDCON.2 (20 nm, different nets) must report
/// unless their two otherwise disjoint stacks meet at BRDL. Each via is wholly
/// inside its landing pads. These deliberately small pads exercise connectivity,
/// not LEF backside geometry rules, which are not implemented by the ICV deck.
fn backside_pair(origin: i32, missing: Option<(i16, usize)>, join_rdl: bool) -> GdsLibrary {
    let mut elems = Vec::new();
    for arm in 0..2 {
        let y = origin + arm as i32 * 35;
        for layer in [SDCON, BPR].into_iter().chain(BACK_METALS) {
            elems.push(rect(layer, [origin, y, origin + 16, y + 16]));
        }
        for layer in [VBPR].into_iter().chain(BACK_VIAS) {
            if missing != Some((layer, arm)) {
                elems.push(rect(layer, [origin + 2, y + 2, origin + 14, y + 14]));
            }
        }
        if !join_rdl {
            elems.push(rect(BRDL, [origin, y, origin + 16, y + 16]));
        }
    }
    if join_rdl {
        elems.push(rect(BRDL, [origin, origin, origin + 16, origin + 51]));
    }
    library(elems)
}

#[rstest]
#[case::intact(None)]
#[case::vbpr(Some(VBPR))]
#[case::bv0(Some(910))]
#[case::bv1(Some(930))]
#[case::bv2(Some(950))]
#[case::bv3(Some(970))]
#[case::bv4(Some(990))]
fn backside_stack_controls_sdcon_different_net_spacing(#[case] missing: Option<i16>) {
    for (origin, tile_um) in [(100, 20.0), (6_990, 7.0), (19_990, 20.0)] {
        for arm in 0..2 {
            let lib = backside_pair(origin, missing.map(|layer| (layer, arm)), true);
            let conn = extract(&lib, tile_um);
            let a = net(&conn, SDCON, origin + 8, origin + 8);
            let b = net(&conn, SDCON, origin + 8, origin + 43);
            assert_eq!(
                a == b,
                missing.is_none(),
                "missing {missing:?} on arm {arm}"
            );
            let violations = gdscheck::run_drc_with_options(
                &lib,
                "gt2n",
                &["sdcon"],
                None,
                "TOP",
                true,
                &gdscheck::RunOptions {
                    tile_um,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(
                violations.len(),
                usize::from(missing.is_some()),
                "{violations:?}"
            );
            assert!(violations.iter().all(|v| v.rule_id == "SDCON.2"));
        }
    }
}

#[test]
fn separate_backside_power_rails_stay_separate() {
    let lib = backside_pair(100, None, false);
    let conn = extract(&lib, 20.0);
    let supply_a = net(&conn, SDCON, 108, 108);
    let supply_b = net(&conn, SDCON, 108, 143);
    assert_ne!(supply_a, supply_b);
    for layer in [BPR].into_iter().chain(BACK_METALS).chain([BRDL]) {
        assert_eq!(net(&conn, layer, 108, 108), supply_a);
        assert_eq!(net(&conn, layer, 108, 143), supply_b);
    }
    let v = gdscheck::run_drc_with(&lib, "gt2n", &["sdcon"], None, "TOP", true).unwrap();
    assert_eq!(
        v.iter().map(|v| v.rule_id.as_str()).collect::<Vec<_>>(),
        ["SDCON.2"]
    );
}

#[rstest]
#[case::no_vias(0)]
#[case::one_via(1)]
#[case::both_vias(2)]
fn crossing_backside_metals_only_connect_where_a_via_is_present(#[case] vias: usize) {
    // Two parallel BM1 supply rails crossed by one BM2 strap. A via at the first
    // crossing must not also attach the other rail just because BM2 overlaps it.
    let mut elems = vec![
        rect(920, [100, 100, 116, 160]),
        rect(920, [135, 100, 151, 160]),
        rect(940, [100, 120, 151, 136]),
    ];
    for i in 0..vias {
        let x = 100 + i as i32 * 35;
        elems.push(rect(930, [x + 2, 122, x + 14, 134]));
    }
    let conn = extract(&library(elems), 20.0);
    let a = net(&conn, 920, 108, 128);
    let b = net(&conn, 920, 143, 128);
    let strap = net(&conn, 940, 125, 128);
    assert_eq!(a == strap, vias >= 1);
    assert_eq!(b == strap, vias == 2);
    assert_eq!(a == b, vias == 2);
}
