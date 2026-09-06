//! Yekaterina compute engine.
//!
//! v1.0.0 shipped as a binary-only crate, which meant `tests/*.rs` had to pull
//! sources in with `#[path = "../src/x.rs"] mod x;` and there was no way to
//! benchmark the engine in-process at all. The v1.1 performance work needs
//! execution-only, validation-only and serialization-only timings, so the crate
//! now also exposes a library target.
//!
//! This is a packaging change only. The module set, their contents and the
//! binary entry point behave exactly as in v1.0.0; `benches/micro.rs` and future
//! tests link against this target instead of textually including sources.
//! The existing v1.0.0 test corpus is deliberately left on its original
//! `#[path]` includes so that it keeps proving what it proved before.

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
pub mod matrix;
pub mod mechanics;
pub mod model;
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
pub mod user_ops;
pub mod vector;
pub mod verification;
pub mod waves;
