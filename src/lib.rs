//! Strata: Git-friendly project archaeology and repository-local memory.
//!
//! The crate is a library plus a thin binary so the command surface and
//! error model can be exercised by tests and reused by later bootstrap
//! tasks.

pub mod cli;
pub mod error;
pub mod repo;
