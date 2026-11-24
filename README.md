# commandagi: the CommandAGI SDK for Rust

The CommandAGI platform from Rust: start agents, launch a simulated world and drive it, read what its camera sees.
And CommandAGI documents declared in Rust: a schematic, a board, a 3D part, a drawing, a workbook, a world. A
document's file is its source of truth; the CommandAGI editor opens it with its usual tools and writes each edit back
into the file as the smallest text edit. No dependencies.

```bash
cargo add commandagi
export COMMANDAGI_API_KEY=cagi_...     # Settings → API keys
```

```rust
use commandagi::client::CommandAGI;
use std::{thread::sleep, time::Duration};

fn main() -> Result<(), commandagi::client::Error> {
    let cagi = CommandAGI::from_env()?; // reads COMMANDAGI_API_KEY

    // Agents: start one on a goal, then steer it.
    let thread = cagi.threads().create("Design a bracket for a NEMA 17 motor")?;
    cagi.threads().send(&thread.id, "Make it 3 mm thick.")?;

    // Embodiments: launch a world YOU drive (no agent in it), watch it, control it.
    let world = cagi.launch("simulation/arm-cell")?; // waits until it declares its controls
    world.sim().run()?;                              // a launched simulation starts paused
    world.sim().ik([0.08, -0.12, 0.86])?;            // the arm's tool point, in metres
    sleep(Duration::from_secs(2));                   // an answer is not a state: let it move, then read
    println!("{}", world.describe()?.text().len());  // where everything is now
    std::fs::write("cell.jpg", world.observe()?.bytes).unwrap(); // what its camera sees: a 320 x 240 JPEG
    world.stop()?;                                   // (dropping the world stops it too)
    Ok(())
}
```

## The platform client

`commandagi::client` is the Python and TypeScript SDKs' client, over the same one door (`POST /mcp`).

| call | what it does |
| --- | --- |
| `CommandAGI::from_env()` / `CommandAGI::new(key)` | a client; `COMMANDAGI_API_KEY`, `COMMANDAGI_BASE_URL` |
| `cagi.threads().create(intent)` / `.create_with(json)` | start an agent on a goal (or with any `create_thread` arguments) |
| `.send(id, text)`, `.events(id)`, `.get(id)`, `.list()`, `.kill(id)`, `.delete(id)` | steer, follow and stop it (`kill` and `delete` need a key of the manager scope) |
| `cagi.launch("simulation/arm-cell")` | launch a world you drive in a new agentless thread; `launch_without_waiting` returns at once |
| `world.controls()`, `world.act(action, payload)` | what the world accepts, and any declared action |
| `world.sim().run()`, `.pause()`, `.step(n)`, `.speed(x)`, `.reset()`, `.ik([x, y, z])`, `.pose(&[(joint, rad)])`, `.grip(close)`, `.stop()` | the typed vocabulary of a simulated world |
| `world.describe()` | the live world: robots, objects, poses |
| `world.observe()`, `.observe_channel("depth")`, `.frames()`, `.observe_changed(&frame, timeout)` | frames: the camera `cam-head` (a 320 x 240 JPEG) and `depth` (a 16-bit PNG, millimetres, 0 where nothing is); a frame is `{ channel_id, width, height, mime, bytes }` |
| `cagi.call(tool, json)` | any platform tool by name |

Frames come over HTTP, so the client opens no socket (the TypeScript and Python SDKs read the thread's WebSocket):
`observe()` is the `observe` tool's latest picture, `observe_channel(id)` the thread's sample route for one channel
(`depth` is only there), and `observe_changed` waits for a frame that differs.

The crate has no dependency, so it sends its requests with the `curl` program that every computer with a shell has:
a request's key and body go to curl on standard input, never in its arguments. For another HTTP client, implement
`client::Transport` (one method) and start with `CommandAGI::with_transport(key, transport)`. An error is
`client::Error { tool, message, detail }`; a call waits 120 s (`.timeout(…)`).

## Documents

Declare a 3D part:

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
module with no imports. (The client starts `curl` only when your program calls it.)

## Documents: one module for each vocabulary

The same documents, element for element, as the TypeScript SDK's JSX (`commandagi/design`) and the Python SDK's
`commandagi.design`.

| module | documents |
| --- | --- |
| `commandagi::design::schematic` | a schematic |
| `commandagi::design::pcb` | a board |
| `commandagi::design::threed` | a 3D part or assembly |
| `commandagi::design::fab` | a machining setup, a slicing setup |
| `commandagi::design::study` | a study (the question of an analysis: `structural`, `flow` or `step-response`) |
| `commandagi::design::testplans` | a test plan |
| `commandagi::design::factory` | a plant in a scene (`check_operations_system`, `machines_for`) |
| `commandagi::design::scene_agents` | the agents of a scene (`check_scene_agents`) |
| `commandagi::design::scene_people` | a person in a scene (`check_look`, `check_goal`, `check_humanoid_policy`, `check_motors`, `check_assist`) |
| `commandagi::design::firmware` | a firmware release, a rollout |
| `commandagi::design::office` | a workbook, a page, a deck |
| `commandagi::design::twod` | a drawing, a painting, a photo, a nest |
| `commandagi::design::media` | a video, a song |
| `commandagi::design::ontology` | a world, a device definition, a dashboard, a geo project (and its shocks, scenarios, models, parameter sets, layers, formulas, views, reports and graphs), a node graph |
| `commandagi::design::business` | a company, an RFC, a case |
| `commandagi::design::classification` | what kind of business or product a thing is: `codes` (`scheme:code`) and `tags`, `classification_problem`, `matches_prefix` |
| `commandagi::design::tasks` | a task, a project |
| `commandagi::design::mounts` | a thread's mounted tasks (`mounts.rs`, in the thread's run folder) |
| `commandagi::design::products` | a product a seller sells (`name.product.rs`): `product`, `variant`, `discount` |
| `commandagi::design::carts` | a buyer's cart at one seller (`Carts/<seller>.cart.rs`): `cart`, `item`, `offer`; no price |
| `commandagi::design::storefronts` | a storefront its holder lists (`<name>.storefront.rs`): `storefront`, `offers`, `offer` |
| `commandagi::design::connectors` | a remote API as a folder tree a mount can list (`name.connector.rs`): `connector`, `credential`, `licence`, `folder`, `request`, `here`, `each().in_(…).as_(…)`, `dir`, `file` |
| `commandagi::design::infra` | a VPN exit (`exit.rs`), a logged-in state (`session.rs`), a vantage (`vantage.rs`). The CommandAGI platform does not run these: its scripts read `vantage.tsx` and `session.tsx`, and `make infra-plan` reads `exit.tsx` |
| `commandagi::design::evals` | an eval (`eval`: its `case`s), a metric (`metric`) |
| `commandagi::design::scripts` | a script (`<name>.script.rs`): its sources, tails, derives, wakes, discovers and judges; with `param()`s, a template |
| `commandagi::design::scouts` | a scout's comparison, a replacement-cost analysis |
| `commandagi::design::agents` | an agent harness, an agent |
| `commandagi::design::records` | a contract, a product instance, a hardware release, a unit's calibration |
| `commandagi::design::postal` | a letter, a postcard |
| `commandagi::design::logistics` | a supplier, a logistics plan |
| `commandagi::design::market` | a market recording |
| `commandagi::design::pdf` | a PDF |

The rules of a call (`src/design/mod.rs` has them in full):

- One constructor for each tag, in snake case (`brush_stroke`, `cap_table`; a Rust keyword gets a trailing `_`:
  `box_`, `type_`). A tag that holds children takes one iterator of elements; a tag that holds text takes the text.
- Attributes are chained methods in snake case of the TypeScript name: `.frozen_rows(1)` sets `frozenRows`. A value is
  a number, a string, a boolean, an array of these, or `json(r#"…"#)` for anything else.
- `commandagi::design::declare(root)` reads the tree and gives the document's graph, or its document beside an empty
  graph. It refuses by name what it cannot read.

`map/` is a second crate (`commandagi-map`, not published): the source map of a document's file, computed with `syn`.
The editors use it to write an edit back into the file.

## What the crate does not have

The Python and TypeScript SDKs have more than this crate does. The client above covers threads and live sessions of a
simulated world; the rest needs more than the standard library.

| | Python | TypeScript | Rust |
| --- | --- | --- | --- |
| document vocabularies (the table above) | yes | yes | yes |
| code-part primitives (`part`, `circuit`, `graph` …) and the CadQuery, JSCAD, replicad and tscircuit importers | CadQuery | JSCAD, replicad, tscircuit | no |
| the platform client: threads, live sessions of a simulated world, any tool by name | yes | yes | yes (`commandagi::client`) |
| the client's typed namespaces for memory, integrations, web, phone, postal … | yes | yes | no: `call(tool, json)` |
| live sessions: frames and typed control | computer, robot, world | computer, robot, world | a simulated world (`sim()`), any action by `act`; frames by `observe` |
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

- The crate is on crates.io (`cargo add commandagi`); `cargo add commandagi@=0.6.10` pins a version. A release is also
  the tag `v<version>` in this repository.
- The three SDKs carry one minor version (0.6). The crate's patch number may be ahead of the other two when only the
  Rust crate changed: 0.6.10 is the first with the client. A vocabulary at version 0.6.x in Rust is the vocabulary at
  0.6.x in Python and TypeScript.
- A version promises nothing about the next one. CommandAGI keeps no compatibility layers: a release may rename or
  remove a function, an element or an attribute, with no alias and no migration. `CHANGELOG.md` says what changed.
- So a document belongs with the version that reads it. Keep the pin in the project that holds the documents, and
  change both together.

## Tests

```bash
cargo test                 # offline: the client runs against a scripted transport and a loopback server
cargo test --manifest-path map/Cargo.toml
```

## Licence

MIT (`LICENSE`).
