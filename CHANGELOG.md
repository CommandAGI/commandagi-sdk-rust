# Changelog

One entry for each release. A release is the tag `v<version>` in this repository. The crate is not on crates.io: a
project pins the tag (README § Versions). The Python and TypeScript SDKs carry the same version number. A version
promises nothing about the next one: read this file before you change the tag you pin.

## Unreleased

- `commandagi::design::threed`: `body().engineering(…)` (material grade, process, finish, vendor, cost), `joint()`
  (`fixed`, `revolute`, `prismatic`, `cylindrical`, `planar`, `screw`, `spherical`, `universal`), `mate()`, `sheet()`
  (an engineering drawing sheet) and `hole().size("M4 close fit")`, each checked against the schema as in the
  TypeScript SDK. A joint's kinds changed name: `hinge` is `revolute`, `slider` is `prismatic`, `ball` is
  `spherical`; joints and mates are their own elements, not fields of the `dynamics` and `assembly` slots.
- `commandagi::design::standards`: `material`, `mass_grams`, `hole_size`, `sized_hole`, `standard_row`: the same
  tables as the TypeScript and Python SDKs, each with where its rows were read.

## 0.5.0 (2026-10-13)

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
