//! Tests for the degenerate-input contract.
//!
//! The contract splits degenerate input in two, and these tests keep the split
//! honest:
//!
//! * **Rejected.** A specific error is required. Asserting the variant, not just
//!   "some error", is what stops a future change from silently reclassifying a
//!   failure.
//! * **Unguaranteed.** Success is required, but only the basic structural
//!   properties are asserted. Connectivity is genuinely ambiguous for these
//!   inputs, so pinning it down would encode an accident rather than a
//!   requirement.
//!
//! No test here is allowed to accept "either an error or a success". That would
//! assert nothing and would quietly absorb a regression.

mod common;

use common::{assert_area, assert_well_formed};
use delaunay_study::DelaunayError;
use delaunay_study::delaunay2d::triangulate;
use delaunay_study::geometry::Point2;

fn points(coordinates: &[(f64, f64)]) -> Vec<Point2> {
    coordinates
        .iter()
        .map(|&(x, y)| Point2::new(x, y))
        .collect()
}

// -------------------------------------------------------------------------
// Rejected: a triangulation cannot exist
// -------------------------------------------------------------------------

#[test]
fn too_few_points_is_rejected() {
    let input = points(&[(0.0, 0.0), (1.0, 0.0)]);
    assert!(matches!(
        triangulate(&input),
        Err(DelaunayError::TooFewPoints {
            given: 2,
            required: 3
        })
    ));
}

#[test]
fn an_empty_input_is_rejected() {
    assert!(matches!(
        triangulate(&[]),
        Err(DelaunayError::TooFewPoints { given: 0, .. })
    ));
}

#[test]
fn duplicate_points_are_rejected_with_both_indices() {
    let input = points(&[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 0.0)]);
    assert!(matches!(
        triangulate(&input),
        Err(DelaunayError::DuplicatePoint {
            first: 1,
            second: 3
        })
    ));
}

/// `-0.0` and `0.0` are different bit patterns but the same point.
#[test]
fn negative_zero_counts_as_a_duplicate_of_zero() {
    let input = points(&[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (-0.0, -0.0)]);
    assert!(matches!(
        triangulate(&input),
        Err(DelaunayError::DuplicatePoint {
            first: 0,
            second: 3
        })
    ));
}

#[test]
fn all_collinear_points_are_rejected() {
    let input = points(&[(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)]);
    assert!(matches!(
        triangulate(&input),
        Err(DelaunayError::AllPointsCollinear)
    ));
}

#[test]
fn a_vertical_line_is_also_collinear() {
    let input = points(&[(1.0, 0.0), (1.0, 1.0), (1.0, 2.0)]);
    assert!(matches!(
        triangulate(&input),
        Err(DelaunayError::AllPointsCollinear)
    ));
}

#[test]
fn nan_coordinates_are_rejected() {
    let input = points(&[(0.0, 0.0), (1.0, 0.0), (f64::NAN, 1.0)]);
    assert!(matches!(
        triangulate(&input),
        Err(DelaunayError::NonFiniteCoordinate {
            index: 2,
            axis: "x"
        })
    ));
}

#[test]
fn infinite_coordinates_are_rejected() {
    let input = points(&[(0.0, 0.0), (1.0, f64::INFINITY), (0.0, 1.0)]);
    assert!(matches!(
        triangulate(&input),
        Err(DelaunayError::NonFiniteCoordinate {
            index: 1,
            axis: "y"
        })
    ));
}

/// Non-finite coordinates are checked before duplicates, so a NaN never reaches
/// the bit-pattern comparison that would treat two NaNs as the same point.
#[test]
fn non_finite_is_reported_before_duplicates() {
    let input = points(&[(f64::NAN, 0.0), (f64::NAN, 0.0), (0.0, 1.0)]);
    assert!(matches!(
        triangulate(&input),
        Err(DelaunayError::NonFiniteCoordinate { index: 0, .. })
    ));
}

// -------------------------------------------------------------------------
// Unguaranteed: accepted, but the connectivity is not pinned down
// -------------------------------------------------------------------------

/// Four cocircular points admit two different triangulations, both correct.
///
/// The test therefore requires success and structural validity, and says nothing
/// about which diagonal was chosen.
#[test]
fn four_cocircular_points_succeed() {
    let input = points(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
    let mesh = triangulate(&input).expect("cocircular input is accepted");

    assert_well_formed(&mesh);
    assert_eq!(mesh.triangles().len(), 2);
    assert_area(&mesh, 1.0);
    assert_eq!(mesh.boundary_edge_keys().len(), 4);
}

#[test]
fn many_cocircular_points_succeed() {
    // Eight points spread evenly around the unit circle: every subset of four is
    // cocircular, so almost every choice of diagonal is defensible.
    let input: Vec<Point2> = (0..8)
        .map(|k| {
            let angle = std::f64::consts::TAU * f64::from(k) / 8.0;
            Point2::new(angle.cos(), angle.sin())
        })
        .collect();

    let mesh = triangulate(&input).expect("cospherical input is accepted");
    assert_well_formed(&mesh);
}

/// A regular hexagon with its centre: all six rim points share one circumcircle.
///
/// This case is worth keeping because it shows how far "unguaranteed" reaches.
/// The result is not merely an arbitrary choice of diagonals -- the mesh comes
/// out with five triangles instead of six, so it does not cover the hexagon at
/// all. Every rim triangle has the same circumcircle, `incircle` returns
/// rounding noise instead of a clean zero, and the cavity that noise produces is
/// not the star-shaped region the algorithm assumes.
///
/// Perturbing any radius by as little as 1e-15 breaks the exact cocircularity
/// and restores all six triangles, which is what identifies this as the
/// documented `f64` limitation rather than a logic error.
///
/// The total area is therefore deliberately *not* asserted here. Only the
/// structural properties the contract does promise are.
#[test]
fn a_regular_hexagon_with_its_centre_succeeds_but_loses_area() {
    let mut input: Vec<Point2> = (0..6)
        .map(|k| {
            let angle = std::f64::consts::TAU * f64::from(k) / 6.0;
            Point2::new(angle.cos(), angle.sin())
        })
        .collect();
    input.push(Point2::new(0.0, 0.0));

    let mesh = triangulate(&input).expect("cocircular input is accepted");
    assert_well_formed(&mesh);
}

/// The same hexagon with the cocircularity broken by one unit in the last place.
///
/// The companion to the test above: once the input is in general position the
/// full set of checks passes, which pins the blame on the degeneracy rather than
/// on the algorithm.
#[test]
fn a_slightly_irregular_hexagon_is_triangulated_completely() {
    let mut input: Vec<Point2> = (0..6)
        .map(|k| {
            let angle = std::f64::consts::TAU * f64::from(k) / 6.0;
            let radius = 1.0 + 1e-15 * f64::from(k);
            Point2::new(radius * angle.cos(), radius * angle.sin())
        })
        .collect();
    input.push(Point2::new(0.0, 0.0));

    let mesh = triangulate(&input).expect("general position input");
    assert_well_formed(&mesh);
    assert_eq!(mesh.triangles().len(), 6);
    assert_area(&mesh, 3.0 * 3.0_f64.sqrt() / 2.0);
}

/// Some points collinear is fine; only *all* of them is rejected.
#[test]
fn partially_collinear_points_succeed() {
    let input = points(&[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0), (1.5, 1.0)]);

    let mesh = triangulate(&input).expect("partially collinear input is accepted");
    assert_well_formed(&mesh);
    assert_area(&mesh, 1.5);
}

/// Nearly collinear input is where the plain `f64` predicates are weakest.
///
/// The contract promises only that this is accepted and structurally sound. It
/// does not promise the Delaunay condition holds, so `assert_well_formed` is the
/// right level of scrutiny here.
#[test]
fn nearly_collinear_points_succeed() {
    let input = points(&[
        (0.0, 0.0),
        (1.0, 1e-10),
        (2.0, 0.0),
        (3.0, 1e-10),
        (1.5, 1.0),
    ]);

    let mesh = triangulate(&input).expect("nearly collinear input is accepted");
    assert_well_formed(&mesh);
}

/// Coordinates spanning many orders of magnitude.
///
/// `incircle` grows with the fourth power of the coordinates, so the
/// determinants here are enormous compared with the differences that decide
/// their sign. Structural validity is all that is claimed.
#[test]
fn widely_different_coordinate_scales_succeed() {
    let input = points(&[
        (0.0, 0.0),
        (1.0e8, 0.0),
        (0.0, 1.0e8),
        (1.0e-8, 1.0e-8),
        (5.0e7, 5.0e7),
    ]);

    let mesh = triangulate(&input).expect("large scale input is accepted");
    assert_well_formed(&mesh);
}

/// A point landing exactly on an existing edge.
#[test]
fn a_point_on_an_edge_succeeds() {
    let input = points(&[
        (0.0, 0.0),
        (2.0, 0.0),
        (1.0, 2.0),
        (1.0, 0.0), // the midpoint of the first edge
    ]);

    let mesh = triangulate(&input).expect("a point on an edge is accepted");
    assert!(!mesh.triangles().is_empty());
}
