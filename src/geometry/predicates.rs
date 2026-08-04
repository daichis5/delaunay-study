//! Geometric predicates, and the sign conventions this crate is built on.
//!
//! # Sign conventions
//!
//! These are fixed for the whole crate. Everything else -- the insertion loop,
//! the tests, the documentation -- assumes them.
//!
//! * [`orient2d`]`(a, b, c) > 0` means `a, b, c` are **counter-clockwise**.
//! * [`incircle`]`(a, b, c, d) > 0` means `d` is **inside** the circumcircle of
//!   `a, b, c` -- **but only when `a, b, c` are counter-clockwise.**
//!
//! That second caveat is the one that bites. `incircle` is a determinant whose
//! sign flips with the orientation of its first three arguments, so calling it
//! on a clockwise triangle silently inverts the answer: points inside the
//! circumcircle report as outside and vice versa. Bowyer-Watson driven by an
//! inverted test does not crash, it just produces a wrong mesh.
//!
//! The defence is an invariant rather than a check at the call site: every
//! triangle stored in a [`crate::delaunay2d::mesh::Mesh2`] is counter-clockwise,
//! and it is made so *at the moment it is created*, not in a final pass.
//!
//! # Floating point
//!
//! Both functions evaluate a determinant in ordinary `f64` arithmetic. The
//! subtractions cancel catastrophically when the points are nearly degenerate,
//! so a result near zero carries no reliable sign.
//!
//! This crate deliberately does **not** paper over that with a tolerance. There
//! is no `if value.abs() < EPSILON { return 0.0 }` here, and callers must not
//! add one either. An absolute epsilon would be wrong anyway: `orient2d` scales
//! with the square of the coordinates and `incircle` with the fourth power, so
//! no single constant is meaningful across both. Instead the raw sign is used,
//! and nearly degenerate input is documented as unguaranteed.
//!
//! Tolerances do exist in this project, but only in *verification* code, where
//! comparing two areas is a genuinely approximate question. They never decide
//! topology.
//!
//! Replacing these with adaptive-precision predicates (Shewchuk's method, or
//! the `robust` crate) is the intended way to lift that limitation later. The
//! signatures are chosen to match, so the swap is local to this module.

use crate::geometry::point2::Point2;

/// Twice the signed area of the triangle `a, b, c`.
///
/// The sign is what callers actually use:
///
/// * `> 0` -- `a, b, c` are counter-clockwise
/// * `< 0` -- `a, b, c` are clockwise
/// * `= 0` -- the three points are collinear
///
/// This is the determinant
///
/// ```text
/// | b.x - a.x   b.y - a.y |
/// | c.x - a.x   c.y - a.y |
/// ```
///
/// # Examples
///
/// ```
/// use delaunay_study::geometry::{orient2d, Point2};
///
/// let a = Point2::new(0.0, 0.0);
/// let b = Point2::new(1.0, 0.0);
/// let c = Point2::new(0.0, 1.0);
/// assert!(orient2d(a, b, c) > 0.0); // counter-clockwise
/// assert!(orient2d(a, c, b) < 0.0); // same triangle, reversed
/// ```
pub fn orient2d(a: Point2, b: Point2, c: Point2) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// Tests `d` against the circumcircle of `a, b, c`.
///
/// **`a, b, c` must be counter-clockwise** (see the module documentation). Given
/// that, the sign means:
///
/// * `> 0` -- `d` lies strictly inside the circumcircle
/// * `< 0` -- `d` lies strictly outside
/// * `= 0` -- the four points are cocircular
///
/// This is the determinant
///
/// ```text
/// | a.x - d.x   a.y - d.y   (a.x - d.x)^2 + (a.y - d.y)^2 |
/// | b.x - d.x   b.y - d.y   (b.x - d.x)^2 + (b.y - d.y)^2 |
/// | c.x - d.x   c.y - d.y   (c.x - d.x)^2 + (c.y - d.y)^2 |
/// ```
///
/// which is the standard lift of the four points onto the paraboloid
/// `z = x^2 + y^2`, translated so that `d` sits at the origin.
///
/// # Examples
///
/// ```
/// use delaunay_study::geometry::{incircle, Point2};
///
/// let a = Point2::new(0.0, 0.0);
/// let b = Point2::new(1.0, 0.0);
/// let c = Point2::new(0.0, 1.0);
///
/// assert!(incircle(a, b, c, Point2::new(0.3, 0.3)) > 0.0); // inside
/// assert!(incircle(a, b, c, Point2::new(2.0, 2.0)) < 0.0); // outside
/// assert_eq!(incircle(a, b, c, Point2::new(1.0, 1.0)), 0.0); // on the circle
/// ```
pub fn incircle(a: Point2, b: Point2, c: Point2, d: Point2) -> f64 {
    let adx = a.x - d.x;
    let ady = a.y - d.y;
    let bdx = b.x - d.x;
    let bdy = b.y - d.y;
    let cdx = c.x - d.x;
    let cdy = c.y - d.y;

    // The lifted third column: squared distance from `d`.
    let alift = adx * adx + ady * ady;
    let blift = bdx * bdx + bdy * bdy;
    let clift = cdx * cdx + cdy * cdy;

    // Cofactor expansion along the first row.
    adx * (bdy * clift - blift * cdy) - ady * (bdx * clift - blift * cdx)
        + alift * (bdx * cdy - bdy * cdx)
}
