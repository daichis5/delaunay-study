//! Coordinates, mesh elements, and geometric predicates.
//!
//! This module holds only the data and the primitive geometric questions. The
//! algorithms that use them live in [`crate::delaunay2d`], keeping the two
//! separable so that the predicates can be studied -- and later replaced with
//! robust versions -- on their own.

pub mod element;
pub mod point2;
pub mod predicates;

pub use element::{Edge, Triangle};
pub use point2::Point2;
pub use predicates::{incircle, orient2d};
