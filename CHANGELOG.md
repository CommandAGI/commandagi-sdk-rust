# Changelog

One entry for each release. A release is the tag `v<version>` in this repository. The crate is not on crates.io: a
project pins the tag (README § Versions). The Python and TypeScript SDKs carry the same version number. A version
promises nothing about the next one: read this file before you change the tag you pin.

## Unreleased

Added

- A bought part drawn as its envelope: `engineering.envelope: true` and its `mountingHoles` (`at`, `axis` in its own
  frame, `diameter`, `depth`). The fasteners check reads those holes, not the envelope's material; a screw off them
  is one `no-mounting-hole` (IC2-13).
- `<library sizes>`: a table of what differs by size (part number, mpn, sku, price breaks, mass), so one `<library>`
  prices every size it lists (CORE-29). The Rust reader now reads a `library` element at all
  (`El::new("library")`), as the TypeScript and Python readers do.

Changed

- A library fastener piece has a mass: its envelope's volume × its stack grade's density (CORE-44). A bill line of
  pieces is named by its designation, and a roll-up line by its part number, not by the first screw's id (CORE-29).

## 0.6.3 (2026-10-21)

The fixes for the microfab team's report on 0.6.2 (110 issues; the ids in brackets are theirs). The vocabulary below
is the same in the three SDKs, in each language's spelling; `@commandagi/host` 0.6.3 carries the checks.

Changed (a document written for 0.6.2 may need an edit)

- One face query for 3D documents, checks and studies: a study's `type` is `geometry`, `nearest` is `farthest` with
  the direction reversed, `radius` and `area` take `{ min, max }`; `at` names the face that holds a point (CORE-24,
  PCBC-12, IC2-08).
- A finding names a body by the id the document gives it (`<body id>`, else the feature that made it); the rebuild's
  `body_0` is `rebuildId` (CORE-13). A covered finding's `severity` is `suppressed`, its own in `coveredSeverity`
  (PCBC-03).
- `commandagi check` prints a compact report (verdict, each check, findings grouped by rule); `--json` or `--out FILE`
  gives the whole report (IC2-12, PCBC-04).
- A `bodies` entry that names no body is an `unknown-body` error: a check of nothing never passes (LABS-02).
- Copper joins where it is drawn, and a pour joins the pads and vias of its net it covers; `from`/`to` are optional
  (EDA-03, EDA-06). A part on the bottom flips its pads, paste, mask, silk and courtyard as KiCad does (EDA-04). An
  unrouted net on a board with copper is an error (EDA-05).
- An unknown material is a warning of the process check (CORE-11).

Added

- Placed parts carry what their file states: engineering, mass, part number, fasteners; a placed assembly keeps its
  members as `<placement>/<id>`. Check, `threed bom`, `threed observe`, STEP and `diff` agree (CORE-01 to -04, -09,
  -20, -26, -27). Bill lines and logistics findings are one per part number with a quantity (LABS-16, IC1-11).
- A `.3d.tsx` imports sibling `.ts`, `.tsx` and `.json` files; a `.3d.py` reads its folder and imports sibling modules
  (CORE-05, CORE-06). Syntax errors name the author's line and column (IC1-01); every refused element in one run
  (IC1-02).
- `commandagi --version` with no key; `check --only ID`, `--timeout S` and a timeout that says so; `release --dry-run`
  and `.releaseignore`; a release also holds the folder's requirements, systems and interfaces (SETUP-01, IC1-09,
  LABS-19, LABS-20, CORE-28). `commandagi layout | route | fabricate <board>` run on this computer (EDA-17).
- Faces: a hole on a face split in two, a pick by a point on the face, a pick among the bodies a feature consumes, an
  ambiguous pick that names its faces (CORE-08, IC2-02, IC2-03, LABS-05). A counterbore deeper than its size's
  (CORE-10); `m2.5` threads (IC2-04); a clearance pair the kernel cannot measure is its own finding (CORE-19).
- Every ISO 286 class (IT01 to IT18, A to ZC) to 3150 mm, and ISO 2768-1 (IC2-05, CORE-17, IC1-06); a fit between two
  sizes of different nominals (CORE-16); a fit's `pair` covers its press fit's clash (CORE-18); `against` on
  interference and clearance (CORE-25, LABS-03); `<accept other>` for a part modelled inside another, an accept sized
  to its finding (PCBC-09, IC1-10); `<hole spare>` and `<permitted diameter>` (IC1-04).
- The mill check judges each face from the setup that reaches it (`make.setups`, or the fewest sides), drilled holes as
  drilled, a plate turned on the table, the machine's materials (CORE-07, -12, -14, -15, -21, -34). The resin printer
  prints: SLA printability (drain, cups) and slicing (CORE-32); `commandagi slicer slice X.slice.tsx` (CORE-33).
- Drawing sheets: hole callouts from the hole features (depth, blind, FAR SIDE), a hole table from a datum corner, no
  overlapping callouts, the release's REV (CORE-22, -31, -36, PCBC-10).
- Studies: inverted elements straightened, each level's reason on its line, `<solver timeout>`, `--keep finest`, a
  fixture on a region (CORE-37, IC2-10, PCBC-11, IC1-15).
- Budgets of any quantity and unit (flow, area, volume …), a priced `<item ref>` to a design with its lead-time path,
  a diff that names every logistics change, process flows (`.flow.tsx`), cost ranges, µm and nm, `other` processes,
  `<library>` data for fastener pieces, `<fastener material partNumber>`, a `chemicals` check (LABS-06 to -12, IC1-03,
  IC1-05, IC1-12, IC2-11, PCBC-13). HFS5-2060 and HFS5-2080 in the T-slot table (CORE-35).
- Boards: pinless parts (EDA-02), four-layer net-class rules (EDA-07), KiCad parity on no-connect pins, datasheets and
  back-side text (EDA-08 to -10), interface sides that read a sheet, a board or a `.sch.py` (EDA-14), a fabrication
  BOM by part number (EDA-19), `near` as a hard rule of the placer (EDA-20).
- Speed: a 1136-hole plate checks in about 5 s (was 4 min 54 s); a 206-body assembly in 45 s (was 4 min 41 s)
  (LABS-04, IC2-06).

In this SDK

- `netclass()` states the rules a four-layer board needs: `.min_width()` (the narrowest track; `.width()` is what the
  router lays), `.edge_clearance()` and `.hole_clearance()` (the least gaps to the board edge and to a hole),
  `.inner(json(r#"{"clearance": 0.15, "width": 0.15}"#))` (the rules on inner copper layers) and `.min_spokes()` (the
  fewest thermal spokes a pad needs to a pour). `commandagi check` holds the board to them and `commandagi export
  kicad` writes them into the project and its `.kicad_dru` (microfab EDA-07).

## 0.6.2 (2026-10-16)

No change in this SDK's code. The release carries `@commandagi/host` 0.6.2, which `commandagi check` and the local 3D
service run:

- A STEP export writes a body's part number: `<body engineering={{ partNumber: "PN-1001" }}>` is the id of the
  body's `PRODUCT` (`PRODUCT('PN-1001','Plate',…)`, as AP242 and AP214 hold it), and the export's `products` list it.
- The `sourcing` check reports `part-number-collision`: two parts under one part number that differ in process,
  material, finish, description, designation, manufacturer or MPN. Before, the cost roll-up merged them silently and
  priced the second as the first. Shape is not compared.

## 0.6.1 (2026-10-15)

Added

- Agents (`commandagi::design::agents`): a harness (`harness.rs`: `harness().kind(…).label(…).provider(…).runs([…])`)
  and an agent (`agent.rs`: a base names its `.harness(…)`, a fork its `.parent(…)`), the same elements as the
  TypeScript SDK's `agents.ts`. A realtime harness says what its model takes and gives live: `.realtime(json(r#"{"in":
  […], "out": […], "audio": {…}, "fps": …, "hz": …, "minutes": …}"#))`.
- Test plans (`commandagi::design::testplans`): a `<name>.testplan.rs` holds the steps that test one serial-numbered
  unit, never the results: `testplan([…]).title(…).device(…)` with `condition()`, `fixture()` and `step()`, the same
  elements as the TypeScript SDK's `testplans.ts`.
- Firmware (`commandagi::design::firmware`): a release (`<name>.firmware.rs`: `firmware([…]).version(…).image(…)
  .sha256(…).sbom(…).device(…)` with `supports().revision(…)`) and a rollout (`<name>.rollout.rs`: `rollout([…])
  .firmware(…).world(…).channel(…).canary(…).waves([…])` with `health()` and `target()`), as the TypeScript SDK's
  `firmware.ts`. A `sha256` that is not 64 lowercase hex digits is refused.
- A world's unit states its `.serial(…)` and its hardware `.revision(…)`.
- Tolerances. `parameter().tolerance(json(…))` takes `{"upper", "lower"}` (mm), `{"class": "H7"}` (an ISO 286 class or
  `ISO 492 bore|outside|width`) or `{"basic": true}`; a tolerance given two ways is refused. `design::tolerances`:
  `tolerance_limits(nominal, &tol)` and `fit_of(&hole, &shaft)`, the classes read from the shipped tables
  (`standards_data.json` `tolerances`). A 3D document's `check` takes `hole`, `shaft`, `intent` (a fit) and `chain`,
  `limit`, `method`, `samples`, `seed` (a stack-up).

Added (a design's logistics, CommandAGI's `docs/formats.md` § logistics, as the TypeScript SDK's)

- A schematic part's logistics: `.suppliers(…)`, `.alternates(…)`, `.datasheet(…)`, `.lifecycle(…)`,
  `.compliance(…)`, `.shipping(…)`, each written as KiCad properties of the part (`LCSC Part`, `LCSC Terms`,
  `Datasheet`, `RoHS`, `HS Code` …) in the TypeScript SDK's forms; `commandagi::design::part_fields` writes them and
  `part_fields()` reads them back. `attrs` may not name a property that an argument says.
- A circuit's `check()` takes `options` (a logistics check's, `{"quantities": [10, 100]}`), kept in `meta.checks`.
- A board's `fab()`: the fab house and its terms, its capabilities, finish, mask, silk, assembly, test, compliance and
  shipping, as one board fact. One per board.
- A 3D document's `item()` (`items`) and `assemble()` (`assemblySteps`); `body().engineering(…)` takes the logistics
  fields and no longer `vendor` or `cost`. The schema reader knows the patterns of a currency, a country and a date.

Added (the vocabulary the TypeScript and Python SDKs have)

- A hardware release and a change (`release()`, `change()`), and the release a unit or an instance was built from. A
  definition names its firmware and revision; `firmware()` also declares the channels a firmware gives.
- Systems (`commandagi::design::systems`): requirements, a system's budgets (a cost budget names the design whose
  logistics roll-up it reads, `.design(…)`) and an interface.
- Studies (`commandagi::design::study`), with the flow kind (`fluid`, `inlet`, `outlet`, `flow_mesh`; units m-kg-s).
- `check()` and `accept()` in a `.sch.rs` and a `.pcb.rs`, kept in the graph's meta (0.6.0 refused them).
- Drawing sheets: section and detail views in the schema; a `sheet()` with a section view and no section, or a bad
  detail, is refused.
- A part's pinout and pads are checked: `.pinout_verified`, `.footprint_verified` and `.package_dimensions` are part
  fields. A material grade states its Poisson's ratio where its source states one.

## 0.6.0 (2026-10-15)

The crate calls no platform tool, so the removal of `pcb_autoroute`, `pcb_optimize_routing` and
`pcb_optimize_placement` from the platform (the Python and TypeScript SDKs' `eda.autoroute`) changes no code here. A
document that a Rust program declares is laid out by the platform's layout tools (`pcb_layout`, `pcb_place_group`,
`pcb_route_nets`).

Added

- A schematic part states its engineering data: `.part_number(…)`, `.manufacturer(…)`, `.mpn(…)`, `.rating(…)` and
  `.attrs(…)` on any part but a ground. Each is one KiCad property of the part (`CommandAGI IPN`, `Manufacturer`,
  `MPN`, `Rating`, and each `attrs` entry by its own name), so the BOM and the board's footprint carry it.
  `PART_FIELDS` names them.
- Declared checks in a 3D document: `check()` (`interference`, `clearance`, `holes`, `fasteners`, `edges`,
  `symmetry`), `accept()` (one finding that is right on purpose, with its reason) and `permitted()` (the findings that
  a process or a purpose permits), and `fastener()`: a screw of the fastener library with its washer and nut, and what
  it clamps into what. Each is checked against the schema, as in the TypeScript SDK.
- A board's layout constraints, each a board fact: `netclass()`, `diffpair()`, `group()`, `keepout()`, `thermal()`,
  `objective()` (`commandagi::design::pcb`; CommandAGI's `docs/eda-layout.md`), as the TypeScript SDK's elements.
- A circuit's checks: `check()` and `accept()` in a schematic and a board (`commandagi::design::schematic`, `pcb`),
  kept in the graph's `meta` (`checks`, `accepted`) as the TypeScript SDK's `<check>` and `<accept>`; a wrong field
  is refused by name.
- `commandagi::design::study`: a study (`.study.rs`), the question of an analysis of a part: `study()`,
  `material()`, `fixture()`, `load_case()`, `force()`, `pressure()`, `mesh()`, `solver()`, `criterion()`,
  `revise()`, the same document as the TypeScript SDK's `study.ts` and the Python SDK's `commandagi.design.study`.
  The `structural` kind; a tag of another kind and a wrong field are refused by name.
- The materials data (`standards_data.json`) carries a grade's `poissonsRatio` where its source states one (ten of
  the 23 grades).

Changed

- The schema reader checks the two `pattern`s the 3D schema now carries (a text that is not blank, an id of
  `A-Z a-z 0-9 _ . -`). It refuses any other pattern by name; it never skips one.
- An example of a task proposal names a simulated run of the world it asks for (`worlds/shop/runs/…`), not a run of
  a second world.

## 0.5.1 (2026-10-15)

Changed

- The README names the local 3D kernel's package: `npm install -g @commandagi/host`, then `commandagi threed
  install`. The crate's code did not change; the version moves with the Python and TypeScript SDKs.

## 0.5.0 (2026-10-14)

The first release with the design layer. 0.5.0 was prepared on 2026-10-13 and not published; this entry holds all of it.

Added

- `commandagi::design::threed`: `body().engineering(…)` (material grade, process, finish, vendor, cost), `joint()`
  (`fixed`, `revolute`, `prismatic`, `cylindrical`, `planar`, `screw`, `spherical`, `universal`), `mate()`, `sheet()`
  (an engineering drawing sheet) and `hole().size("M4 close fit")`, each checked against the schema as in the
  TypeScript SDK. A joint's kinds changed name: `hinge` is `revolute`, `slider` is `prismatic`, `ball` is
  `spherical`; joints and mates are their own elements, not fields of the `dynamics` and `assembly` slots.
- `commandagi::design::standards`: `material`, `mass_grams`, `hole_size`, `sized_hole`, `standard_row`: the same
  tables as the TypeScript and Python SDKs, each with where its rows were read.
- 3D documents: a hole's `positions` and `sketch`; faces and edges chosen by a query, `{"kind", "select": {…}}`,
  beside the pick; a part is built in the order of its elements (a feature that names no `consumes` follows every
  body-making feature written before it).

Changed

- A union's refusal names the field at fault (`edges/0/select/vertical: is not one of its fields`).


The first tagged release. The crate was `0.1.0` in its manifest from its first commit (2026-10-05) and through the 56
commits to this one.

- `commandagi::design`: a document declared in Rust, one module for each vocabulary: `schematic`, `pcb`, `threed`,
  `fab`, `office`, `twod`, `media`, `ontology`, `business`, `tasks`, `monitors`, `records`, `postal`, `pdf`.
- `commandagi::abi`: what the CommandAGI code sandbox calls in the compiled WebAssembly module.
- `map/`: the source map of a document's file (each element's call and its arguments), computed with `syn`, for the
  editors' write-back.
- No dependencies, no build script, no procedural macros.
- `README.md`, `LICENSE` (MIT, as `Cargo.toml` states) and this file.

Not in this crate (README § What the crate does not have): the platform client, live sessions, the 3D kernel's client
and the command line.
