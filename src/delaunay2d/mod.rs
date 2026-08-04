//! Delaunay triangulation of a point set in the plane.
//!
//! [`triangulate`] is the entry point. The surrounding types live in
//! [`mesh`], and the algorithm itself in [`bowyer_watson`].

pub mod bowyer_watson;
pub mod mesh;

pub use bowyer_watson::triangulate;
pub use mesh::Mesh2;
