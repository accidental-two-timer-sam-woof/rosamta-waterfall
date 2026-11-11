//! Platform-independent configuration and deployment logic.
#![deny(clippy::expect_used, clippy::unwrap_used)]

pub mod diag;
pub mod fs;
pub mod host;
pub mod layout;
pub mod name;
pub mod path;

pub mod config;
pub mod docedit;
pub mod resolve;
pub mod state;
pub mod template;

pub mod exec;
pub mod plan;
pub mod preflight;
pub mod recovery;

pub mod apply;
pub mod init;
pub mod manage;
pub mod query;
