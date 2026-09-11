//! Yekaterina compute engine.
//!
//! v1.0.0 shipped as a binary-only crate, which meant `tests/*.rs` had to pull
//! sources in with `#[path = "../src/x.rs"] mod x;` and there was no way to
//! benchmark the engine in-process at all. The v1.1 performance work needs
//! execution-only, validation-only and serialization-only timings, so the crate
//! now also exposes a library target.
//!
//! v1.3 keeps the promoted v1.2 registry and engine source files intact as
//! historical baselines, then places explicit aggregate/dispatch shims in front
//! of them. v1.4 follows the same pattern: the complete v1.3 surface remains an
//! explicit layer while a small math/discovery layer is placed in front of it.

pub mod advanced_matrix;
pub mod advanced_numerical;
pub mod advanced_probability;
pub mod advanced_signal;
pub mod advanced_stats;
pub mod algebra;
pub mod astronomy;
pub mod chemistry;
pub mod color;
pub mod complex_math;
pub mod curve;
pub mod data_ops;
pub mod deep_linalg;
pub mod discrete;
pub mod electrical;
#[path = "engine.rs"]
pub mod engine_v12;
#[path = "engine_v13.rs"]
pub mod engine_v13;
#[path = "engine_v14.rs"]
pub mod engine;
pub mod engineering;
pub mod extra_math;
pub mod fluids;
pub mod formula;
pub mod frame;
pub mod geodesy;
pub mod geometry;
pub mod inference;
pub mod information;
pub mod limits;
pub mod math_v14;
pub mod matrix;
pub mod mechanics;
pub mod model;
pub mod multiplicity;
pub mod networking;
pub mod numerical;
pub mod ode;
pub mod optics;
pub mod optimization;
pub mod physics;
pub mod pool;
pub mod practical;
pub mod precision;
pub mod predicate;
pub mod probability;
pub mod radix;
#[path = "registry.rs"]
pub mod registry_v12;
#[path = "registry_v13.rs"]
pub mod registry_v13;
#[path = "registry_v14.rs"]
pub mod registry;
pub mod safety;
pub mod scheduler;
pub mod series;
pub mod server;
pub mod signal;
pub mod special_functions;
pub mod stats;
pub mod storage;
pub mod thermodynamics;
pub mod time_ops;
pub mod transformer;
pub mod transformer_candidate;
pub mod transformer_winding;
pub mod user_ops;
pub mod vector;
pub mod verification;
pub mod waves;
