// SPDX-FileCopyrightText: 2026 aesc silicon
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Routing patterns, M1-M6 and V1-V5. The odd metals run along y and the even ones
//! along x, and each metal's six rules read the same way on its own axes: `.1` its
//! width, `.2` its length, `.3` the space across its wires, `.4` the space between
//! their tips, `.5` the space corner to corner, `.6` a bend. So one builder draws every
//! metal, along `u` and across `v`, and turns the result onto the metal's axes.
//!
//! A via is flush with the metal running along y across x, and with the one running
//! along x across y, as V*.3 and V*.4 ask; each metal runs past it along its length,
//! as the two enclosure rules ask.

use super::{Corpus, DBU, Rows, bx, corner_rise, pg};
use gds21::GdsElement;

const OFFSET: f64 = 5000.0;

pub(super) fn generate(c: &Corpus<'_>) {
    for (n, vals) in [
        (1, [14.0, 28.0, 14.0, 20.0, 16.0]),
        (2, [12.0, 24.0, 12.0, 17.0, 15.0]),
        (3, [14.0, 28.0, 14.0, 20.0, 16.0]),
        (4, [21.0, 42.0, 21.0, 30.0, 24.0]),
        (5, [21.0, 42.0, 21.0, 30.0, 24.0]),
        (6, [38.0, 55.0, 38.0, 55.0, 45.0]),
    ] {
        metal(c, n, vals);
    }
    for v in [
        Via::new(1, (14.0, 12.0), 4.0, 18.0),
        Via::new(2, (14.0, 12.0), 4.0, 18.0),
        Via::new(3, (14.0, 21.0), 4.0, 18.0),
        Via::new(4, (21.0, 21.0), 5.0, 27.0),
        Via::new(5, (21.0, 38.0), 6.0, 27.0),
    ] {
        via(c, &v);
    }
}

/// A metal's frame: along `u`, across `v`, which for a metal running along y are y and x.
struct Frame {
    layer: (i16, i16),
    along_y: bool,
}

impl Frame {
    fn xy(&self, (u, v): (f64, f64)) -> (f64, f64) {
        if self.along_y { (v, u) } else { (u, v) }
    }

    /// A box `du` long and `dv` across, its corner at (u, v).
    fn bx(&self, u: f64, v: f64, du: f64, dv: f64) -> GdsElement {
        let (x, y) = self.xy((u, v));
        let (w, h) = self.xy((du, dv));
        bx(self.layer, x, y, w, h)
    }

    fn pg(&self, pts: &[(f64, f64)]) -> GdsElement {
        let pts: Vec<_> = pts.iter().map(|&p| self.xy(p)).collect();
        pg(self.layer, &pts)
    }
}

fn metal(c: &Corpus<'_>, n: u8, [w, len, s, t, k]: [f64; 5]) {
    let deck = format!("m{n}");
    let name = format!("M{n}");
    let id = |r: u8| format!("M{n}.{r}");
    let w = c.src(&id(1), "min_width", w);
    let len = c.src(&id(2), "min_width", len);
    let s = c.src(&id(3), "min_space", s);
    assert_eq!(c.src(&id(3), "min_notch", s), s);
    let t = c.src(&id(4), "min_space", t);
    assert_eq!(c.src(&id(4), "min_notch", t), t);
    let k = c.src(&id(5), "min_space", k);
    let f = Frame {
        layer: c.layer(&name),
        along_y: n % 2 == 1,
    };
    let long = 300.0;
    let write =
        |r: u8, case: &str, elems| c.write(&deck, &format!("{}.{case}", id(r)), true, elems);

    let mut rows = Rows::new(OFFSET);
    let elems = [w, w - DBU]
        .map(|ww| f.bx(OFFSET, rows.next(ww), long, ww))
        .to_vec();
    write(1, "narrow", elems);

    let mut rows = Rows::new(OFFSET);
    let elems = [len, len - DBU]
        .map(|l| f.bx(OFFSET, rows.next(w), l, w))
        .to_vec();
    write(2, "short", elems);

    let mut rows = Rows::new(OFFSET);
    let mut elems = vec![];
    for gap in [s, s - DBU] {
        let v = rows.next(2.0 * w + gap);
        elems.push(f.bx(OFFSET, v, long, w));
        elems.push(f.bx(OFFSET, v + w + gap, long, w));
    }
    write(3, "close", elems);

    // A U: two wires `gap` apart, joined at one end by a bar as long as `.2` asks.
    let mut rows = Rows::new(OFFSET);
    let mut elems = vec![];
    for gap in [s, s - DBU] {
        let (u, v) = (OFFSET, rows.next(2.0 * w + gap));
        let top = v + 2.0 * w + gap;
        elems.push(f.pg(&[
            (u, v),
            (u + long, v),
            (u + long, v + w),
            (u + len, v + w),
            (u + len, top - w),
            (u + long, top - w),
            (u + long, top),
            (u, top),
        ]));
    }
    write(3, "notch", elems);

    let mut rows = Rows::new(OFFSET);
    let mut elems = vec![];
    for gap in [t, t - DBU] {
        let v = rows.next(w);
        elems.push(f.bx(OFFSET, v, long, w));
        elems.push(f.bx(OFFSET + long + gap, v, long, w));
    }
    write(4, "close", elems);

    // Two wires tip to tip, joined by a third along their sides: a slot `gap` long.
    let mut rows = Rows::new(OFFSET);
    let mut elems = vec![];
    for gap in [t, t - DBU] {
        let (u, v) = (OFFSET, rows.next(2.0 * w));
        let u1 = u + long + gap;
        elems.push(f.pg(&[
            (u, v),
            (u + long, v),
            (u + long, v + w),
            (u1, v + w),
            (u1, v),
            (u1 + long, v),
            (u1 + long, v + 2.0 * w),
            (u, v + 2.0 * w),
        ]));
    }
    write(4, "notch", elems);

    // Two wires, the second past the first's tip along and beside it across, so only
    // their corners face: `du` along, then the least rise across that clears `.5`.
    let du = (k * 0.6 / DBU).round() * DBU;
    let dv = corner_rise(k, du);
    let mut rows = Rows::new(OFFSET);
    let mut elems = vec![];
    for rise in [dv, dv - DBU] {
        let v = rows.next(2.0 * w + rise);
        elems.push(f.bx(OFFSET, v, long, w));
        elems.push(f.bx(OFFSET + long + du, v + w + rise, long, w));
    }
    write(5, "corner", elems);

    // A wire, then one jogged a DBU across half way along.
    let mut rows = Rows::new(OFFSET);
    let mut elems = vec![f.bx(OFFSET, rows.next(w), long, w)];
    let (u, v) = (OFFSET, rows.next(w + DBU));
    let m = u + long / 2.0;
    elems.push(f.pg(&[
        (u, v),
        (m, v),
        (m, v + DBU),
        (u + long, v + DBU),
        (u + long, v + w + DBU),
        (m, v + w + DBU),
        (m, v + w),
        (u, v + w),
    ]));
    write(6, "bent", elems);
}

/// One via level: Vn, from Mn to Mn+1.
struct Via {
    n: u8,
    /// The via's least width across x and height across y.
    size: (f64, f64),
    /// What each metal runs past it along its length.
    en: f64,
    corner: f64,
}

impl Via {
    fn new(n: u8, size: (f64, f64), en: f64, corner: f64) -> Self {
        Via {
            n,
            size,
            en,
            corner,
        }
    }

    /// The metal running along y (the odd one) and the one along x.
    fn metals(&self) -> (u8, u8) {
        if self.n % 2 == 1 {
            (self.n, self.n + 1)
        } else {
            (self.n + 1, self.n)
        }
    }

    fn id(&self, r: &str) -> String {
        format!("V{}.{r}", self.n)
    }
}

fn via(c: &Corpus<'_>, v: &Via) {
    let deck = format!("v{}", v.n);
    let (my, mx) = v.metals();
    let wx = c.src(&v.id("1"), "min_width", v.size.0);
    let wy = c.src(&v.id("2"), "min_width", v.size.1);
    let en_y = c.src(&v.id(&format!("M{my}.EN")), "min_enclosure", v.en);
    let en_x = c.src(&v.id(&format!("M{mx}.EN")), "min_enclosure", v.en);
    let k = c.src(&v.id("6"), "min_space", v.corner);
    let (cut, ly, lx) = (
        c.layer(&format!("V{}", v.n)),
        c.layer(&format!("M{my}")),
        c.layer(&format!("M{mx}")),
    );
    let run = 20.0;
    let tall = wy + 2.0 * run;
    // A `w`×`h` via at (x, y), the metal along y flush across x and `b` and `t` past it,
    // the metal along x flush across y and `l` and `r` past it; `None` leaves one out.
    let stack = |x: f64,
                 y: f64,
                 (w, h): (f64, f64),
                 along_y: Option<(f64, f64)>,
                 along_x: Option<(f64, f64)>| {
        let mut e = vec![bx(cut, x, y, w, h)];
        if let Some((b, t)) = along_y {
            e.push(bx(ly, x, y - b, w, b + h + t));
        }
        if let Some((l, r)) = along_x {
            e.push(bx(lx, x - l, y, l + w + r, h));
        }
        e
    };
    let full = Some((run, run));
    let ok = |x: f64, y: f64| stack(x, y, (wx, wy), full, full);
    let write =
        |r: &str, case: &str, elems| c.write(&deck, &format!("{}.{case}", v.id(r)), true, elems);

    let mut rows = Rows::new(OFFSET);
    let elems = [wx, wx - DBU]
        .into_iter()
        .flat_map(|w| stack(OFFSET, rows.next(tall), (w, wy), full, full))
        .collect();
    write("1", "narrow", elems);

    let mut rows = Rows::new(OFFSET);
    let elems = [wy, wy - DBU]
        .into_iter()
        .flat_map(|h| stack(OFFSET, rows.next(tall), (wx, h), full, full))
        .collect();
    write("2", "narrow", elems);

    // A via whose metal along y sits a DBU to the right: one vertical wall lies outside
    // it and the other inside it, both off its outline; then one whose metal is a DBU
    // wider each side, both walls inside it.
    let mut rows = Rows::new(OFFSET);
    let mut elems = ok(OFFSET, rows.next(tall));
    let y = rows.next(tall);
    elems.extend(stack(OFFSET, y, (wx, wy), None, full));
    elems.push(bx(ly, OFFSET + DBU, y - run, wx, tall));
    let y = rows.next(tall);
    elems.extend(stack(OFFSET, y, (wx, wy), None, full));
    elems.push(bx(ly, OFFSET - DBU, y - run, wx + 2.0 * DBU, tall));
    write("3", "align", elems);

    // The same across y for the metal along x.
    let mut rows = Rows::new(OFFSET);
    let mut elems = ok(OFFSET, rows.next(tall));
    let y = rows.next(tall);
    elems.extend(stack(OFFSET, y, (wx, wy), full, None));
    elems.push(bx(lx, OFFSET - run, y + DBU, wx + 2.0 * run, wy));
    let y = rows.next(tall);
    elems.extend(stack(OFFSET, y, (wx, wy), full, None));
    elems.push(bx(
        lx,
        OFFSET - run,
        y - DBU,
        wx + 2.0 * run,
        wy + 2.0 * DBU,
    ));
    write("4", "align", elems);

    // Two vias corner to corner, each in its own wires, run out to the enclosures.
    let dx = (k * 0.6 / DBU).round() * DBU;
    let dy = corner_rise(k, dx);
    let mut rows = Rows::new(OFFSET);
    let mut elems = vec![];
    let tight = (Some((en_y, en_y)), Some((en_x, en_x)));
    for rise in [dy, dy - DBU] {
        let y = rows.next(2.0 * wy + rise);
        elems.extend(stack(OFFSET, y, (wx, wy), tight.0, tight.1));
        elems.extend(stack(
            OFFSET + wx + dx,
            y + wy + rise,
            (wx, wy),
            tight.0,
            tight.1,
        ));
    }
    write("6", "corner", elems);

    // `.7` and `.8`: a via with its lower (upper) metal missing, then with it abutting
    // the via's side - the left side for the metal along y, the bottom for the other -
    // which shares no area with it.
    for (r, m) in [("7", v.n), ("8", v.n + 1)] {
        let mut rows = Rows::new(OFFSET);
        let mut elems = ok(OFFSET, rows.next(tall));
        let y = rows.next(tall);
        let y2 = rows.next(tall);
        if m == my {
            elems.extend(stack(OFFSET, y, (wx, wy), None, full));
            elems.extend(stack(OFFSET, y2, (wx, wy), None, full));
            elems.push(bx(ly, OFFSET - wx, y2 - run, wx, tall));
        } else {
            elems.extend(stack(OFFSET, y, (wx, wy), full, None));
            elems.extend(stack(OFFSET, y2, (wx, wy), full, None));
            elems.push(bx(lx, OFFSET - run, y2 - wy, wx + 2.0 * run, wy));
        }
        write(r, "off", elems);
    }

    let mut rows = Rows::new(OFFSET);
    let elems = [(en_y, en_y), (en_y - DBU, en_y), (en_y, en_y - DBU)]
        .into_iter()
        .flat_map(|bt| stack(OFFSET, rows.next(tall), (wx, wy), Some(bt), full))
        .collect();
    write(&format!("M{my}.EN"), "enc", elems);

    let mut rows = Rows::new(OFFSET);
    let elems = [(en_x, en_x), (en_x - DBU, en_x), (en_x, en_x - DBU)]
        .into_iter()
        .flat_map(|lr| stack(OFFSET, rows.next(tall), (wx, wy), full, Some(lr)))
        .collect();
    write(&format!("M{mx}.EN"), "enc", elems);
}
