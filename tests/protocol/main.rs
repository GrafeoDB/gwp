//! Protocol tests against a stateful, recording backend over real sockets.
//!
//! - `values`: every GQL value type round-trips client to backend to client
//! - `catalog`: schema, graph and graph type flows, including gwp#15
//! - `sessions`: session properties and resets reach the backend
//! - `transactions`: lifecycle, failures and races
//! - `robustness`: untrusted input, unknown sessions, concurrency, streaming
//! - `session_required`: with authentication, no RPC is served without a
//!   live session; without it, catalog, admin and search name their session
//! - `end_to_end`: the `GqlServer` builder and the high-level client

mod common;

mod catalog;
mod end_to_end;
mod robustness;
mod session_required;
mod sessions;
mod transactions;
mod values;
