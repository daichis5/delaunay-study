//! Tests for the geometric predicates, and above all for their sign conventions.

use delaunay_study::geometry::{Point2, incircle, orient2d};

/// The reference counter-clockwise triangle used throughout this file.
fn ccw_triangle() -> (Point2, Point2, Point2) {
    (
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(0.0, 1.0),
    )
}

#[test]
fn orient2d_is_positive_for_counter_clockwise() {
    let (a, b, c) = ccw_triangle();
    assert!(orient2d(a, b, c) > 0.0);
}

#[test]
fn orient2d_is_negative_for_clockwise() {
    let (a, b, c) = ccw_triangle();
    assert!(orient2d(a, c, b) < 0.0);
}

#[test]
fn orient2d_is_zero_for_collinear_points() {
    let a = Point2::new(0.0, 0.0);
    let b = Point2::new(1.0, 1.0);
    let c = Point2::new(2.0, 2.0);
    assert_eq!(orient2d(a, b, c), 0.0);
}

#[test]
fn orient2d_equals_twice_the_signed_area() {
    let a = Point2::new(0.0, 0.0);
    let b = Point2::new(4.0, 0.0);
    let c = Point2::new(0.0, 3.0);
    // Area 6, so the determinant is 12.
    assert_eq!(orient2d(a, b, c), 12.0);
}

#[test]
fn orient2d_is_invariant_under_cyclic_rotation() {
    let (a, b, c) = ccw_triangle();
    assert_eq!(orient2d(a, b, c), orient2d(b, c, a));
    assert_eq!(orient2d(b, c, a), orient2d(c, a, b));
}

#[test]
fn incircle_is_positive_inside() {
    let (a, b, c) = ccw_triangle();
    // The circumcircle is centred at (0.5, 0.5) with radius sqrt(0.5).
    assert!(incircle(a, b, c, Point2::new(0.5, 0.5)) > 0.0);
    assert!(incircle(a, b, c, Point2::new(0.3, 0.3)) > 0.0);
}

#[test]
fn incircle_is_negative_outside() {
    let (a, b, c) = ccw_triangle();
    assert!(incircle(a, b, c, Point2::new(2.0, 2.0)) < 0.0);
    assert!(incircle(a, b, c, Point2::new(-1.0, -1.0)) < 0.0);
}

#[test]
fn incircle_is_zero_on_the_circle() {
    let (a, b, c) = ccw_triangle();
    // The fourth corner of the unit square is exactly on the circumcircle.
    assert_eq!(incircle(a, b, c, Point2::new(1.0, 1.0)), 0.0);
}

/// The convention that the rest of the crate depends on.
///
/// `incircle` is a determinant, so swapping two of its first three arguments
/// negates it. A clockwise triangle therefore reports every inside point as
/// outside. Bowyer-Watson driven by that would build a wrong mesh without ever
/// failing loudly, which is why the mesh keeps a counter-clockwise invariant
/// instead of checking orientation at each call site.
#[test]
fn incircle_sign_inverts_for_a_clockwise_triangle() {
    let (a, b, c) = ccw_triangle();
    let inside = Point2::new(0.4, 0.4);

    assert!(orient2d(a, b, c) > 0.0);
    assert!(incircle(a, b, c, inside) > 0.0);

    // Same three points, reversed winding.
    assert!(orient2d(a, c, b) < 0.0);
    assert!(incircle(a, c, b, inside) < 0.0);

    assert_eq!(incircle(a, b, c, inside), -incircle(a, c, b, inside));
}

/// Rotating the first three arguments is an even permutation of the
/// determinant's rows, so the value is mathematically unchanged.
///
/// In `f64` it comes out only *nearly* unchanged: the cofactor expansion adds
/// the same terms in a different order, and floating point addition is not
/// associative. Asserting exact equality here would fail. What does agree is the
/// sign, which is the only thing the algorithm ever consumes -- and the reason
/// the crate is careful to depend on signs rather than magnitudes.
#[test]
fn incircle_is_invariant_under_cyclic_rotation() {
    let (a, b, c) = ccw_triangle();
    let d = Point2::new(0.4, 0.4);

    let values = [
        incircle(a, b, c, d),
        incircle(b, c, a, d),
        incircle(c, a, b, d),
    ];

    for value in values {
        assert!(value > 0.0, "{d:?} is inside, so every rotation must agree");
    }

    let largest = values.iter().cloned().fold(f64::MIN, f64::max);
    let smallest = values.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        largest - smallest < 1e-12 * largest.abs(),
        "rotations disagree by more than rounding: {values:?}"
    );
}

/// A demonstration of the limitation the crate documents rather than hides.
///
/// These three points are collinear in exact arithmetic, so `orient2d` should be
/// zero. In `f64` the subtractions cancel and leave rounding noise, whose sign
/// is arbitrary. Nothing here asserts a sign; the point is that the magnitude is
/// tiny while the answer is not trustworthy, which is exactly why nearly
/// degenerate input is documented as unguaranteed.
#[test]
fn orient2d_loses_its_sign_near_degeneracy() {
    let a = Point2::new(0.5, 0.5);
    let b = Point2::new(12.0, 12.0);
    let c = Point2::new(24.0, 24.0);
    assert_eq!(orient2d(a, b, c), 0.0);

    // Perturb one coordinate by a single unit in the last place.
    let nudged = Point2::new(f64::from_bits(c.x.to_bits() + 1), c.y);
    let value = orient2d(a, b, nudged);
    assert!(
        value.abs() < 1e-13,
        "expected a value swamped by rounding, got {value}"
    );
}
