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

The backside stack, BV0 to BRDL, connects to the buried power rail from below:

.. code-block:: text

   BRDL --BV4-- BM4 --BV3-- BM3 --BV2-- BM2 --BV1-- BM1 --BV0-- BPR --VBPR-- SDCON

These are distinct GDS layers; projected overlap between two metals connects them
only through a declared via. The connect graph includes the whole stack, so
``SDCON.2`` applies its different-net spacing after following backside connections.
Separate power rails remain separate unless geometry actually connects them.

**BV0 through BRDL have connectivity support only.** The ``main`` suite includes
all implemented checks, but does not check backside metal or via width, spacing,
enclosure, routing grids or end-of-line constraints. The existing ``bpr`` and
``vbpr`` decks still check the buried rail and its connection to SDCON.

Backside rule sources
~~~~~~~~~~~~~~~~~~~~~

The audit uses GT2N revision ``699e2c10b2b494a120745515f59df8815f4f0175``:

* The `IC Validator runset
  <https://github.com/azadnaeemi/GT2N/blob/699e2c10b2b494a120745515f59df8815f4f0175/icv_runset/gt2_main.drc.rs>`_
  declares the backside layers and connect steps, but its included rule files
  contain no checks for BV0, BM1–BM4, BV1–BV4 or BRDL.
* The `technology LEF
  <https://github.com/azadnaeemi/GT2N/blob/699e2c10b2b494a120745515f59df8815f4f0175/techlib/gt2_tech.lef>`_
  supplies routing and cut-layer declarations, fixed via templates and generated-via
  enclosure constraints. These have not been translated into gdscheck decks.

The LEF's ``WIDTH`` and ``SPACING`` declarations are listed below in nm. This table
records the source values; it is not a table of checks implemented by gdscheck.

.. list-table::
   :header-rows: 1

   * - Layers
     - GDS layer / datatype
     - WIDTH
     - SPACING
   * - BV0
     - 910 / 0
     - 32
     - 32
   * - BM1, BM2
     - 920 / 0, 940 / 0
     - 56
     - 56
   * - BV1–BV4
     - 930 / 0, 950 / 0, 970 / 0, 990 / 0
     - 56
     - 56
   * - BM3, BM4
     - 960 / 0, 980 / 0
     - 360
     - 360
   * - BRDL
     - 1000 / 0
     - 1600
     - 1600

BM1–BM4 also carry ``LEF58_EOLKEEPOUT``, ``LEF58_CORNERSPACING`` and
``LEF58_RECTONLY`` properties. The ``BVia0Array`` through ``BVia4Array`` generators
specify directional enclosures and cut-array spacing. Fixed templates must be
distinguished from general rules: for example, ``BV0_0`` gives BPR a 1 nm horizontal
margin around its cut, while ``BVia0Array`` asks for 10 nm. Also, ``BV4_0`` draws a
1600 × 360 nm cut despite BV4's 56 nm ``WIDTH`` declaration. An eventual backside
geometry deck needs to resolve these semantics and template differences explicitly;
the frontside metal and via limits are not substituted for them.


Tests
-----

``tests/gt2n.rs`` runs the full suite over the 13 nm SVT library, vendored under
``tests/data/gt2n/static/`` (BSD-3-Clause). It reports two issues that are the
library's own. ``gt2_6t_mux2_x1`` lands a V0 flush with the end of its M1, and on the
13 nm sheets ``gt2_6t_decapcc`` draws two VGs 5 nm from an active. The other nine
libraries give the same result, apart from the 13 nm-only VG finding.

Every rule in every deck also has a generated pattern, written by ``gen/gt2n`` under
``tests/data/gt2n/generated/<deck>/`` (``just gen-testdata-for gt2n``). Each pattern
draws the shape exactly at the runset's limit, then one 0.5 nm DBU past it, inside
whatever else its deck reads: a VG in its gate and M0, a V0 flush with both wires.
The generator reads its limits from ``icv_runset/Include/gt2.*.drc.rs`` and stops if a
deck disagrees, including a deck GT2N shares with GT3. The test names the count each
pattern reports, and any other rule the drawing must also break, such as the bend in
a notch.

That standard-cell fixture contains no BV0–BRDL geometry. Separate synthetic tests
exercise two SDCONs joined only through the entire backside stack. Removing each
via in turn, on either branch, must disconnect the contacts and make ``SDCON.2``
report their 19 nm gap. These cases run across 7 µm and 20 µm tile boundaries.
Additional tests keep separate power rails apart and check two BM1 rails crossed
by BM2 with zero, one or two BV1s. The synthetic pads test connectivity, not
compliance with the LEF geometry declarations above.
