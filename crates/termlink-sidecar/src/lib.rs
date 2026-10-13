//! arc-011 PH-1 sidecar library.
//!
//! B2 scope: `estate` (C-1 guard) and `testkit` (C-12 core).

pub mod estate;
pub mod testkit;
pub mod wire;
pub mod store;
pub mod api;
pub mod send;
pub mod pull;
pub mod policy;
pub mod frame;
pub mod adapter;
pub mod deliver;
pub mod hook;
pub mod daemon;
pub mod client;
pub mod cli;
pub mod deadline;
pub mod beat;
