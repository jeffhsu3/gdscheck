.. SPDX-FileCopyrightText: 2026 aesc silicon
..
.. SPDX-License-Identifier: AGPL-3.0-or-later

GT3
===


Overview
--------

``gt3`` is the 3 nm GAAFET PDK from the Georgia Institute of Technology (initial
release). The decks translate its IC Validator runset (``icv_runset/gt3_main.drc.rs``
and the files it includes) rule for rule. There is one deck per included file, and the
ids are the ones the runset's rule descriptions give. Layouts are read at the real
3 nm scale on a 0.5 nm grid.

The suites are ``main`` (every deck), ``feol`` (through V0, what a standard cell holds)
and ``beol`` (M1 to M6). The runset has no rules for V6 and above, and neither has
this PDK.


Translation
-----------

* ICV's ``direction = HORIZONTAL`` reads ``facing: x``, ``VERTICAL`` reads
  ``facing: y``, and ``external_corner1`` reads ``facing: none``.
* ``enclose`` is a ``min_enclosure`` read where the two layers interact.
* An exact width (``internal1 == v``, then the layer ``not`` the result) is an
  ``exact_width`` on that axis.
* SDCON.2, the tip-to-tip space between contacts on different nets, is net-aware. It
  runs on the runset's own connect database, declared in ``pdk.yml``.

Each deck's header lists the rules the runset names but does not check. GATE.2, an
exact 27 nm gate space, is checked as a minimum, which is how the runset effectively
reads it too. Upstream typos are corrected: ``BRP.1`` is ``BPR.1``, and ``V4.M3.EN``
is ``V4.M5.EN``.


Tests
-----

``tests/gt3.rs`` runs the full suite over the 6-track RVT library, vendored under
``tests/data/gt3/static/`` (BSD-3-Clause) with every cell abutted into rows. The only
violation reported is M0.4, between gt3_6t_nor3_x2_rvt and gt3_6t_oa211_x1_rvt.
