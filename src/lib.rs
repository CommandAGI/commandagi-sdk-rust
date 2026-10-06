//! THE COMMANDAGI SDK FOR RUST — CommandAGI documents declared as code that is their source of truth.
//!
//! A document's `.rs` file declares it with the vocabulary of one family module of [`design`] (a schematic:
//! [`design::schematic`]). The CommandAGI editor runs it: the local host compiles it to WebAssembly against this crate,
//! offline, and the sandbox runs it with no imports. The editor shows the document the file declares and writes each
//! edit made there back into the file as the smallest text edit.
//!
//! ```
//! use commandagi::design::schematic::*;
//!
//! fn document() -> El {
//!     group("Divider", [
//!         voltagesource("V1").voltage("9").sch_x(114.3).sch_y(114.3),
//!         resistor("R1").resistance("3k").sch_x(114.3).sch_y(88.9),
//!         trace(".V1 > .pos", ".R1 > .pin1"),
//!     ])
//! }
//! # let graph = commandagi::design::declare(document()).unwrap().graph().unwrap();
//! # assert_eq!(graph.get("id").and_then(|v| v.as_str()), Some("eda:Divider"));
//! ```

pub mod abi;
pub mod design;
