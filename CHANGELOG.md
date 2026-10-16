# Changelog

One entry for each release. A release is the tag `v<version>` in this repository. The crate is not on crates.io: a
project pins the tag (README § Versions). The Python and TypeScript SDKs carry the same version number. A version
promises nothing about the next one: read this file before you change the tag you pin.

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
