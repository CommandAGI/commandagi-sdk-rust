# commandagi: the CommandAGI SDK for Rust

CommandAGI documents declared in Rust: a schematic, a board, a 3D part, a drawing, a workbook, a world. The file is
the document's source of truth. The CommandAGI editor opens it with its usual tools and writes each edit back into
the file as the smallest text edit.

```toml
[dependencies]
commandagi = { git = "https://github.com/CommandAGI/commandagi-sdk-rust", tag = "v0.6.8" }
```

```rust
use commandagi::design::threed::*;

fn document() -> El {
    part([
        box_().id("plate").center([0, 0, 4]).size([60, 40, 8]).operation("new"),
        cylinder().id("hole").center([0, 0, -1]).axis([0, 0, 1]).radius(3).height(10).operation("cut").consumes(["plate"]),
    ])
    .name("Plate")
}

fn main() {
    // The document's graph as JSON: what the editor stores, and what the 3D design service builds.
    let graph = commandagi::design::declare(document()).unwrap().graph().unwrap();
    println!("{}", graph.text());
}
```

The crate has no dependencies, no build script and no procedural macros. The CommandAGI local host compiles a
document's `.rs` file against this crate alone, offline, to WebAssembly (`commandagi::abi`), and its sandbox runs the
module with no imports.

## Documents: one module for each vocabulary

The same documents, element for element, as the TypeScript SDK's JSX (`commandagi/design`) and the Python SDK's
`commandagi.design`.

| module | documents |
| --- | --- |
| `commandagi::design::schematic` | a schematic |
| `commandagi::design::pcb` | a board |
| `commandagi::design::threed` | a 3D part or assembly |
| `commandagi::design::fab` | a machining setup, a slicing setup |
| `commandagi::design::study` | a study (the question of an analysis of a part) |
| `commandagi::design::testplans` | a test plan |
| `commandagi::design::firmware` | a firmware release, a rollout |
| `commandagi::design::office` | a workbook, a page, a deck |
| `commandagi::design::twod` | a drawing, a painting, a photo, a nest |
| `commandagi::design::media` | a video, a song |
| `commandagi::design::ontology` | a world, a device definition, a dashboard, a geo project, a node graph |
| `commandagi::design::business` | a company, an RFC, a case |
| `commandagi::design::tasks` | a task, a project |
| `commandagi::design::mounts` | a thread's mounted tasks (`mounts.rs`, in the thread's run folder) |
| `commandagi::design::evals` | an eval (`eval`: its `case`s), a metric (`metric`) |
| `commandagi::design::monitors` | a monitor |
| `commandagi::design::agents` | an agent harness, an agent |
| `commandagi::design::records` | a contract, a product instance |
| `commandagi::design::postal` | a letter, a postcard |
| `commandagi::design::pdf` | a PDF |

The rules of a call (`src/design/mod.rs` has them in full):

- One constructor for each tag, in snake case (`brush_stroke`, `cap_table`; a Rust keyword gets a trailing `_`:
  `box_`, `type_`). A tag that holds children takes one iterator of elements; a tag that holds text takes the text.
- Attributes are chained methods in snake case of the TypeScript name: `.frozen_rows(1)` sets `frozenRows`. A field no
  method names (a port of your own node type) is set by the generic `.with_attr("cutoff_hz", 6)`, which takes the
  TypeScript name; the source map reads it as that attribute. A value is a number, a string, a boolean, an array of
  these, or `json(r#"…"#)` for anything else.
- `commandagi::design::declare(root)` reads the tree and gives the document's graph, or its document beside an empty
  graph. It refuses by name what it cannot read.

`map/` is a second crate (`commandagi-map`, not published): the source map of a document's file, computed with `syn`.
The editors use it to write an edit back into the file.

## What the crate does not have

The Python and TypeScript SDKs have these. This crate does not, because each one needs the network and so a
dependency, and the local host must compile a document against this crate with none.

| | Python | TypeScript | Rust |
| --- | --- | --- | --- |
| document vocabularies (the table above) | yes | yes | yes |
| the agent, harness and infrastructure vocabularies | no | yes | no |
| code-part primitives (`part`, `circuit`, `graph` …) and the CadQuery, JSCAD, replicad and tscircuit importers | CadQuery | JSCAD, replicad, tscircuit | no |
| the platform client (threads, memory, integrations, web, phone, postal …) | yes | yes | no |
| live sessions (frames and typed control of a computer, a robot or a simulated world) | yes | yes | no |
| the 3D kernel's client (`observe`, `export`, `measure`) | `commandagi.kernel` | `Kernel` | no |
| a command line (`commandagi code eval`, `commandagi threed serve`, `commandagi daemon`) | no | yes | no |

To build a 3D document written in Rust, print its graph (the example above) and post it to the 3D design service
with an HTTP client of your choice. On this computer, with no account:

```bash
npm install -g @commandagi/host && commandagi threed install          # once
COMMANDAGI_THREED_TOKEN=<a long random text> commandagi threed serve --port 8091
cargo run > plate.json
curl -s http://127.0.0.1:8091/export -H "authorization: Bearer $COMMANDAGI_THREED_TOKEN" \
  -H 'content-type: application/json' -d "{\"format\":\"step\",\"docText\":$(jq -Rs . < plate.json)}" | jq -r .content > plate.step
```

Or open the `.rs` file in a CommandAGI editor: the local host compiles it and the editor builds it.

## Versions

- A release is the tag `v<version>` in this repository. The crate is not on crates.io. Pin the tag, as above; a
  branch moves under you.
- The three SDKs carry one version number. A vocabulary at version X in Rust is the vocabulary at version X in
  Python and TypeScript.
- A version promises nothing about the next one. CommandAGI keeps no compatibility layers: a release may rename or
  remove a function, an element or an attribute, with no alias and no migration. `CHANGELOG.md` says what changed.
- So a document belongs with the version that reads it. Keep the pin in the project that holds the documents, and
  change both together.

## Tests

```bash
cargo test                 # 104 tests, offline
cargo test --manifest-path map/Cargo.toml
```

## Licence

MIT (`LICENSE`).
