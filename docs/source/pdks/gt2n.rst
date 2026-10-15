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
The runset stops at M6; ``main`` and ``beol`` go on to RDL with the decks below, from
the technology LEF.

Two of GT2N's own rules are read more closely than the runset does:

* GATE.2, an exact 28 nm gate space, is read as the 42 nm gate pitch it makes with
  GATE.1's 14 nm gate: every gate is centred on a multiple of 42 nm, as in all ten
  libraries, and no two are closer than 28 nm. A design drawn in another frame sets
  the rule's ``offset``. A gate missing from a row is reported too: two gates facing
  each other across x within 1 µm must be 28 nm apart. Gates further apart read as
  separate blocks, and a row's ends are never read. An empty site in a row is a
  missing gate, which ``gt2_6t_filler`` closes.
* ACT.6 requires every part of an active's end to lie in a DUMMY. The runset reports
  only an end that touches no DUMMY at all.


Above M6: the tech LEF
----------------------

The runset checks nothing from V6 up. The decks ``v6`` to ``v13``, ``m7`` to ``m13`` and
``rdl`` take their rules from the technology LEF (``techlib/gt2_tech.lef``, at the
revision below) instead, and every id is marked ``.LEF``, so a marker says which
source it answers to.

Each metal reads as the runset reads M6, because M6's LEF block states the same rules
as the runset does, with one exception: ``.1`` is the ``WIDTH``, ``.2`` the length from
``MINSIZE``, ``.3`` the ``SPACING`` across the wires, ``.4`` the tip-to-tip space, the
``LEF58_EOLKEEPOUT``'s forward extension, ``.5`` the ``LEF58_CORNERSPACING``, and
``.6`` no bend, the ``LEF58_RECTONLY``. The exception is M6's length, 55 nm in the
runset (M6.2) and 57 nm in the LEF's ``MINSIZE`` and ``AREA``; M6 keeps the runset's.
Values in nm:

.. list-table::
   :header-rows: 1

   * - Layer
     - Direction
     - ``.1`` width
     - ``.2`` length
     - ``.3`` space
     - ``.4`` tip
     - ``.5`` corner
   * - M7, M9
     - vertical
     - 38
     - 57
     - 38
     - 55
     - 45
   * - M8
     - horizontal
     - 38
     - 57
     - 38
     - 55
     - 45
   * - M10, M11
     - horizontal, vertical
     - 56
     - none
     - 56
     - 80
     - 66
   * - M12, M13
     - horizontal, vertical
     - 360
     - none
     - 360
     - 450
     - 450
   * - RDL
     - horizontal
     - 1600
     - none
     - 1600
     - none
     - none

M10 to M13 have no ``MINSIZE`` or ``AREA``, so no ``.2``. RDL has only a ``WIDTH`` and
a ``SPACING``: its ``.3`` is read every way, and it has no ``.4``, ``.5`` or ``.6``.
``.4`` reads every tip, as M6.4 does; the LEF's keep-out reads only a line end
narrower than its ``EOLKEEPOUT`` width (39, 57 and 361 nm), which every
minimum-width wire is.

A via's size is the cut its ``V<n>_0`` template and ``Via<n>Array`` rule draw, not
the cut layer's ``WIDTH``, as V5.1 and V5.2 hold V5 to its 21 × 38 nm template and
not its 21 nm ``WIDTH``. The two disagree from V9 up: V9's cut is 38 × 56 nm, V11's
56 × 360, V12's 360 × 360 and V13's 360 × 1600, where each ``WIDTH`` is 38 or 56 nm.
Each via has:

* ``.1`` and ``.2``, its width across x and y: the template's cut.
* ``.5``, the cut layer's ``SPACING``, read every way: 38 nm for V6 to V9, 56 nm for
  V10 to V13. The runset's via space (V1-V5's ``.6``) has no LEF value, and
  ``Via<n>Array``'s ``SPACING`` is a cut pitch equal to the cut, so it says nothing.
* ``.7`` and ``.8``, the cut inside its lower and its upper metal. The LEF's
  ``ENCLOSURE`` of 0 across a metal requires the cut inside it, which is stricter than
  the runset's V5.7 and V5.8, where the cut only has to touch. The LEF does not require
  a cut's walls on its metals' outlines, so the runset's ``.3`` and ``.4`` have no
  counterpart.
* ``V<n>.M<k>.EN.LEF``, each metal's enclosure along its length, from
  ``Via<n>Array``: 8 nm for V6 to V8, 10 for V9, 12 for V10 and V11, 20 for V12, and
  for V13, 40 by M13 and 200 by RDL.

The LEF's routing ``PITCH`` is not a check here, as M1 to M6 have none.

Run over OpenROAD-flow-scripts' routed ``gt2n/aes``, which routes up to M9, these
decks report nothing.


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

The 13 nm and 31 nm sheets share one placement site, but not one ACT: the 13 nm cells
draw it at y 35–48 nm and the 31 nm cells at y 26–57 nm. The filler and the backside
tap (``tapbspdn``) are one 42 nm pitch wide and draw their ACT across the whole cell,
so a 31 nm one between 13 nm cells stands 9 nm proud of them above and below. Each
protruding piece is one pitch across and fails ACT.2's 84 nm, as it would upstream. A
13 nm one-pitch cell is clean beside either sheet. The decap, two pitches wide, stands
proud by exactly 84 nm and is clean from either sheet.

A flow that mixes the sheets has to pick its one-pitch cells with this in mind.
OpenROAD's fill insertion takes the first filler of each width listed in
``FILL_CELLS``. ORFS's ``gt2n`` platform loads both sheets, 31 nm first; its
``FILL_CELLS`` listed every sheet's filler and its ``TAP_CELL_NAME`` followed the
primary sheet, so every filler and tap was a 31 nm cell. On ``gt2n/aes``, with mostly
13 nm logic, that gave 9,480 ACT.2 markers, every one on a 31 nm filler or tap between
13 nm cells. Taking the fillers and the tap from the 13 nm sheet whenever it is
loaded clears them all and leaves synthesis, placement and routing unchanged. GT2N's
own example flow names ``gt2_6t_filler_w31_lvt`` as its filler, so a 13 nm design
built from it should set ``FILL_CELLS`` to the matching 13 nm filler. The
``ACT.2.sheets`` pattern draws all four filler-and-neighbour pairings.

Every rule in every deck also has a generated pattern, written by ``gen/gt2n`` under
``tests/data/gt2n/generated/<deck>/`` (``just gen-testdata-for gt2n``). Each pattern
draws the shape exactly at the runset's limit, then one 0.5 nm DBU past it, inside
whatever else its deck reads: a VG in its gate and M0, a V0 flush with both wires.
The generator reads its limits from ``icv_runset/Include/gt2.*.drc.rs``, and above M6
from ``techlib/gt2_tech.lef``, and stops if a deck disagrees, including a deck GT2N
shares with GT3. The test names the count each
pattern reports, and any other rule the drawing must also break, such as the bend in
a notch.

That standard-cell fixture contains no BV0–BRDL geometry. Separate synthetic tests
exercise two SDCONs joined only through the entire backside stack. Removing each
via in turn, on either branch, must disconnect the contacts and make ``SDCON.2``
report their 19 nm gap. These cases run across 7 µm and 20 µm tile boundaries.
Additional tests keep separate power rails apart and check two BM1 rails crossed
by BM2 with zero, one or two BV1s. The synthetic pads test connectivity, not
compliance with the LEF geometry declarations above.
