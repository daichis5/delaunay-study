//! An educational, from-scratch Delaunay triangulation.
//!
//! * [`geometry`] -- points, elements, and the geometric predicates
//! * [`error`] -- the single error type, and the rule for what becomes an error
//!
//! See the README for the implemented scope.

pub mod error;
pub mod geometry;

pub use error::{DelaunayError, Result};
