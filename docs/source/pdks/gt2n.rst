.. SPDX-FileCopyrightText: 2026 aesc silicon
..
.. SPDX-License-Identifier: AGPL-3.0-or-later

GT2N
====


Overview
--------

``gt2n`` is the 2 nm nanosheet PDK with backside power delivery from the Georgia
Institute of Technology (initial release). Its IC Validator runset
(``icv_runset/gt2_main.drc.rs``) is :doc:`gt3`'s with seven values changed: a 14 nm
gate on a 42 nm pitch, a 16 nm SDCON and VBPR, and an 8 nm active. It also adds a deck
for the five threshold-voltage layers (ELVT, ULVT, SVT, HVT and SRAMVT; LVT is the
default and has no layer).

The PDK ``extends: gt3``. The decks that changed live under ``pdks/gt2n/decks/``,
and the rest are GT3's files. The suites match GT3's: ``main``, ``feol`` and ``beol``.


Backside power
--------------

The backside stack, BV0 to BRDL, connects to the buried power rail from below. The
runset includes it in the connect database but has no DRC rules for it, and neither
does this PDK. Its layers are declared so that net extraction, and SDCON.2's
different-net space, see the backside nets.


Tests
-----

``tests/gt2n.rs`` runs the full suite over the 13 nm SVT library, vendored under
``tests/data/gt2n/static/`` (BSD-3-Clause). It reports two issues that are the
library's own. ``gt2_6t_mux2_x1`` lands a V0 flush with the end of its M1, and on the
13 nm sheets ``gt2_6t_decapcc`` draws two VGs 5 nm from an active. The other nine
libraries give the same result, apart from the 13 nm-only VG finding.
