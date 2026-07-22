//! Strata: Git-friendly project archaeology and repository-local memory.
//!
//! The crate is a library plus a thin binary so the command surface and
//! error model can be exercised by tests and reused by later bootstrap
//! tasks.

pub mod artifact;
pub mod cli;
pub mod doctor;
pub mod edges;
pub mod error;
pub mod fortune;
pub mod read;
pub mod repo;
pub mod transition;
