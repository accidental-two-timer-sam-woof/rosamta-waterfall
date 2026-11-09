//! Feasibility pilot for the rosamta detailed design (docs/05-detailed-design).
//!
//! Each module answers one pilot question; the tests under `tests/` hold the
//! evidence. The implementation does not reuse this code.

pub mod atomic;
pub mod bound;
pub mod link;
pub mod payload;
pub mod platform;
