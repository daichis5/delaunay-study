//! An educational, from-scratch Delaunay triangulation.
//!
//! The crate is split so that each idea can be read on its own:
//!
//! * [`geometry`] -- points, elements, and the geometric predicates
//! * [`delaunay2d`] -- the 2D Bowyer-Watson algorithm and its mesh type
//! * [`io`] -- CSV input and output
//! * [`error`] -- the single error type, and the rule for what becomes an error
//!
//! # Start here
//!
//! Two documents carry the reasoning that the code only implements:
//!
//! * [`geometry::predicates`] explains the sign conventions and why a tolerance
//!   is deliberately absent from them.
//! * [`delaunay2d::triangulate`] states exactly which inputs are rejected and
//!   which are accepted with an unguaranteed result.
//!
//! # Scope
//!
//! Only the 2D case is implemented. The 3D tetrahedralization, and the Python
//! verification and visualization scripts, are not part of this version. See the
//! README for what is planned.
//!
//! ```
//! use delaunay_study::delaunay2d::triangulate;
//! use delaunay_study::geometry::Point2;
//!
//! let square_with_center = vec![
//!     Point2::new(0.0, 0.0),
//!     Point2::new(1.0, 0.0),
//!     Point2::new(1.0, 1.0),
//!     Point2::new(0.0, 1.0),
//!     Point2::new(0.5, 0.5),
//! ];
//!
//! let mesh = triangulate(&square_with_center)?;
//! assert_eq!(mesh.triangles().len(), 4);
//! assert!((mesh.total_area() - 1.0).abs() < 1e-12);
//! # Ok::<(), delaunay_study::DelaunayError>(())
//! ```

pub mod delaunay2d;
pub mod error;
pub mod geometry;
pub mod io;

pub use error::{DelaunayError, Result};
