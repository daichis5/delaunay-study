//! Property checks shared by the integration tests.
//!
//! Connectivity equality is a weak test for a Delaunay triangulation, because
//! cocircular input admits several equally valid answers. These checks assert
//! *properties* instead, so they stay meaningful on input whose exact
//! connectivity is not pinned down.
//!
//! # Tolerances
//!
//! The constants here are verification tolerances and nothing else. They compare
//! areas and classify a point against a circle -- genuinely approximate
//! questions. They must never be pushed down into
//! `delaunay_study::geometry::predicates`, whose signs decide topology and are
//! used raw on purpose.
//!
//! Both are absolute and assume input on roughly a unit scale, which every
//! caller of the stricter checks satisfies. `incircle` grows with the fourth
//! power of the coordinates, so a fixed constant would be meaningless on input
//! spanning many orders of magnitude; such cases only get the basic checks.

#![allow(dead_code)]

use std::collections::HashSet;

use delaunay_study::delaunay2d::Mesh2;
use delaunay_study::geometry::{Point2, incircle, orient2d};

/// Tolerance for comparing areas.
pub const AREA_TOLERANCE: f64 = 1e-9;

/// How far inside a circumcircle a point must be before it counts as a violation.
///
/// A point exactly on a circumcircle is not a violation: the Delaunay condition
/// is that no point is *strictly* inside. Cocircular input, such as the corners
/// of a square, would otherwise fail this check.
pub const EMPTY_CIRCUMCIRCLE_TOLERANCE: f64 = 1e-9;

/// Checks that hold for any accepted input, including the degenerate cases whose
/// connectivity this crate does not guarantee.
pub fn assert_well_formed(mesh: &Mesh2) {
    assert!(
        !mesh.triangles().is_empty(),
        "triangulation produced no elements"
    );

    for triangle in mesh.triangles() {
        let [a, b, c] = triangle.nodes;

        assert!(
            a < mesh.vertices().len() && b < mesh.vertices().len() && c < mesh.vertices().len(),
            "triangle {triangle:?} references a vertex outside 0..{}",
            mesh.vertices().len()
        );
        assert!(
            a != b && b != c && c != a,
            "triangle {triangle:?} repeats a vertex"
        );

        let [pa, pb, pc] = mesh.triangle_points(triangle);
        let orientation = orient2d(pa, pb, pc);
        assert!(
            orientation > 0.0,
            "triangle {triangle:?} is not counter-clockwise (orient2d = {orientation})"
        );
    }
}

/// The full set of checks, for input in general position.
///
/// Adds the global properties that only hold when the mesh is a genuine
/// triangulation of the convex hull.
pub fn assert_valid_triangulation(mesh: &Mesh2, expected_area: f64) {
    assert_well_formed(mesh);
    assert_all_points_used(mesh);
    assert_edge_use_counts_valid(mesh);
    assert_empty_circumcircle(mesh);
    assert_area(mesh, expected_area);
}

/// Every input point must appear in at least one triangle.
///
/// Catches points silently dropped by a broken cavity, which an area comparison
/// alone can miss.
pub fn assert_all_points_used(mesh: &Mesh2) {
    let used = mesh.used_vertices();
    let missing: Vec<_> = (0..mesh.vertices().len())
        .filter(|index| !used.contains(index))
        .collect();
    assert!(
        missing.is_empty(),
        "input points {missing:?} appear in no triangle"
    );
}

/// Every edge must be used once (on the hull) or twice (inside the mesh).
///
/// This is what makes the area check trustworthy: an overlap and a hole of equal
/// size cancel in the total area, but both show up here.
pub fn assert_edge_use_counts_valid(mesh: &Mesh2) {
    for (edge, count) in mesh.edge_use_counts() {
        assert!(
            count == 1 || count == 2,
            "edge {edge:?} is used by {count} triangles, expected 1 or 2"
        );
    }
}

/// No vertex may lie strictly inside any triangle's circumcircle.
///
/// This is the Delaunay condition itself. It is quadratic in the mesh size, so
/// only small fixed cases and modest random cases use it.
pub fn assert_empty_circumcircle(mesh: &Mesh2) {
    for triangle in mesh.triangles() {
        let [a, b, c] = mesh.triangle_points(triangle);
        for (index, &point) in mesh.vertices().iter().enumerate() {
            if triangle.contains(index) {
                continue;
            }
            let value = incircle(a, b, c, point);
            assert!(
                value <= EMPTY_CIRCUMCIRCLE_TOLERANCE,
                "vertex {index} lies inside the circumcircle of {triangle:?} (incircle = {value})"
            );
        }
    }
}

/// The triangle areas must sum to the area of the region being triangulated.
pub fn assert_area(mesh: &Mesh2, expected: f64) {
    let total = mesh.total_area();
    assert!(
        (total - expected).abs() < AREA_TOLERANCE,
        "total area {total} differs from the expected {expected}"
    );
}

/// The boundary edges, as a sorted list of normalized edge keys.
pub fn boundary_edges(mesh: &Mesh2) -> Vec<(usize, usize)> {
    mesh.boundary_edge_keys()
}

/// The area of the convex hull of `points`.
pub fn convex_hull_area(points: &[Point2]) -> f64 {
    let hull = convex_hull(points);

    // Shoelace formula over the counter-clockwise hull.
    let mut twice_area = 0.0;
    for i in 0..hull.len() {
        let p = hull[i];
        let q = hull[(i + 1) % hull.len()];
        twice_area += p.x * q.y - q.x * p.y;
    }
    twice_area / 2.0
}

/// The convex hull of `points`, counter-clockwise, via the monotone chain algorithm.
///
/// Written out rather than derived from the mesh, so that comparing the two is
/// an independent check rather than a tautology.
pub fn convex_hull(points: &[Point2]) -> Vec<Point2> {
    let mut sorted = points.to_vec();
    sorted.sort_by(|p, q| {
        p.x.partial_cmp(&q.x)
            .unwrap()
            .then(p.y.partial_cmp(&q.y).unwrap())
    });

    let build = |input: &[Point2]| -> Vec<Point2> {
        let mut chain: Vec<Point2> = Vec::new();
        for &point in input {
            while chain.len() >= 2
                && orient2d(chain[chain.len() - 2], chain[chain.len() - 1], point) <= 0.0
            {
                chain.pop();
            }
            chain.push(point);
        }
        chain.pop();
        chain
    };

    let mut hull = build(&sorted);
    let reversed: Vec<Point2> = sorted.iter().rev().copied().collect();
    hull.extend(build(&reversed));
    hull
}

/// A small linear congruential generator, so that random tests need no dependency.
///
/// Seeded explicitly by every test, so failures are always reproducible.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// A value in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        // Constants from Knuth's MMIX.
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // The high bits of an LCG are the well-behaved ones.
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
    }

    /// `count` points uniformly spread over the unit square.
    pub fn points_in_unit_square(&mut self, count: usize) -> Vec<Point2> {
        let mut points = Vec::with_capacity(count);
        let mut seen = HashSet::new();
        while points.len() < count {
            let point = Point2::new(self.next_f64(), self.next_f64());
            if seen.insert(point.exact_key()) {
                points.push(point);
            }
        }
        points
    }
}
