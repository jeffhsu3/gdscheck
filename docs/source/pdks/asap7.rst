.. SPDX-FileCopyrightText: 2026 aesc silicon
..
.. SPDX-License-Identifier: AGPL-3.0-or-later

ASAP7
=====


Overview
--------

``asap7`` is the 7 nm predictive FinFET PDK from Arizona State University and ARM,
release 1p7. The rules are the design rule manual's (``asap7_drm_201207a.pdf``). The
Calibre deck the manual was written against is not public, so the manual is the
reference. The KLayout port at `laurentc2/ASAP7_for_KLayout
<https://github.com/laurentc2/ASAP7_for_KLayout>`_ was read alongside it for what each
rule measures.

Layouts are read at the real 7 nm scale on a 0.25 nm grid (4000 DBU per µm), which is
how the standard-cell libraries ship. An older layout drawn 4× up for Calibre must be
scaled down first.

Three suites:

``main``
   Every deck.
``feol``
   The front end and middle of line, through V0: what a standard cell holds.
``beol``
   The routing stack, M1 to the pad.

The decks are per layer, named after it in lower case (``well``, ``fin``, ``gate``,
``active``, ``gcut``, ``select``, ``sdt``, ``lisd``, ``lig``, ``v0``, ``m1`` … ``m9``,
``v1`` … ``v9``), plus ``geometry`` for the manual's non-orthogonal-shape rule.


One-way layers
--------------

Most ASAP7 layers route in one direction, so the manual gives them one width and space
across the direction and another along it: "minimum horizontal width of WELL", "minimum
vertical spacing of ACTIVE", and M4–M7 each on its own axis. These rules read
``facing: x`` (vertical walls, a horizontal span) or ``facing: y`` on
:doc:`../checks/min_width`, :doc:`../checks/min_space`, :doc:`../checks/min_notch` and
:doc:`../checks/min_enclosure`. Where the manual names a corner-to-corner space
(``M1.S.6``, ``LIG.LISD.S.7``, the V0–V3 ``S.2``), the rule reads ``facing: none``,
which is the pairs that face each other nowhere under the value.

A via "enclosed on at least two opposite sides" is ``sides: opposite``. "V1 must be
the same width as M2" is a ``max_enclosure`` of nothing on ``sides: opposite``, which
means the via is flush with the wire on one pair of sides.

The M1–M3, LISD and LIG spacings depend on the lengths of the two facing edges: side to
side (both over 36 nm), tip to side, and tip to tip. They are read on edge layers
sorted by length (``edge_layers`` in ``pdk.yml``), as are the M8/M9 spacings, which
switch at 80 nm.


What is not checked
-------------------

Each deck's header lists the rules of its section it does not check, and why. In
summary:

* Grids and tracks: ``GATE.S.1`` (the 54 nm poly pitch), ``M4.AUX.1``, ``M4.AUX.2`` and
  ``M4.AUX.4`` and their M5–M7 twins. The manual places the tracks by an offset set per
  design.
* Width quantisation: ``ACTIVE.W.2`` and ``SDT.W.3`` (whole multiples of 27 nm), and
  ``M4.W.3``/``M4.W.4`` to ``M7.W.3``/``M7.W.4`` (no even multiple of the minimum).
* Net-aware or reach rules: ``ACTIVE.S.2A`` (source/drain on different nets) and
  ``ACTIVE.LUP.1`` (30 µm latch-up reach).
* Endcap-dependent via spacings: ``V0.S.3``/``V0.S.4`` to ``V3.S.3``/``V3.S.4``. The
  corner-to-corner rule reads the loosest of them, 23 nm.
* ``GATE.AUX.2``, ``V0.LISD.EN.3``, ``V0.LIG.AUX.2``, the SDT vertical overlaps (which
  the SDT edge and cover rules already bound), ``M4.S.4``/``M4.S.5`` and their M5–M7
  twins.

``FIN.S.1`` is an exact 27 nm pitch. The deck checks only its lower bound, a space of
at least 20 nm.


Readings
--------

A few rules are read differently from the manual's literal wording, because the
shipped standard-cell libraries (verified with the Calibre deck) contradict that
wording:

* ``V0.LIG.EN.4`` is worded as the LIG's enclosure by the V0. The libraries draw both
  that case (a V0 over a 16 nm LIG rail, 1 nm past it on each side) and a V0 flush with
  the end of a wider LIG. The rule reads the LIG walls inside the via and skips the
  flush one.
* ``SDT.ACTIVE.AUX.2``, "SDT horizontal edges must coincide with ACTIVE horizontal
  edges", is read as "an SDT does not end inside its ACTIVE". Where an ACTIVE steps,
  the libraries run the SDT on past it.
* The select, VT and well minimum widths fire on the 54 nm half-width filler and tap
  cells when these are checked on their own. Placed between other cells, they are part
  of a wider shape.

The KLayout port reports several rules the libraries pass under this deck. Its
``M1.S.2``, ``LIG.S.4-5`` and ``LIG.LISD.S.7`` read a corner-to-corner gap as a
tip-to-side or side-to-side one, and its ``V0.S.1`` holds every corner-to-corner pair
to 27 nm.


Tests
-----

``tests/asap7.rs`` runs the full suite over the whole ASAP7 LVT standard-cell library,
vendored under ``tests/data/asap7/static/`` (BSD-3-Clause). It asserts that only the
library's own known violations are reported.
