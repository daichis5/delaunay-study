//! An educational, from-scratch Delaunay triangulation.
//!
//! * [`geometry`] -- points, elements, and the geometric predicates
//! * [`delaunay2d`] -- the 2D Bowyer-Watson algorithm and its mesh type
//! * [`error`] -- the single error type, and the rule for what becomes an error
//!
//! See the README for the implemented scope.

pub mod delaunay2d;
pub mod error;
pub mod geometry;

pub use error::{DelaunayError, Result};
