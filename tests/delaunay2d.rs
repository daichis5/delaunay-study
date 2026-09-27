//! Tests for the 2D triangulation on input in general position.
//!
//! Degenerate and rejected input lives in `degeneracy.rs`.

mod common;

use common::{Lcg, assert_valid_triangulation, assert_well_formed, convex_hull, convex_hull_area};
use delaunay_study::delaunay2d::triangulate;
use delaunay_study::geometry::Point2;

fn points(coordinates: &[(f64, f64)]) -> Vec<Point2> {
    coordinates
        .iter()
        .map(|&(x, y)| Point2::new(x, y))
        .collect()
}

#[test]
fn three_points_give_one_triangle() {
    let input = points(&[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]);
    let mesh = triangulate(&input).expect("a single triangle is valid input");

    assert_eq!(mesh.triangles().len(), 1);
    assert_valid_triangulation(&mesh, 0.5);
}

#[test]
fn vertices_are_returned_unchanged_and_in_order() {
    let input = points(&[(0.0, 0.0), (2.0, 0.0), (1.5, 1.0), (0.4, 0.9)]);
    let mesh = triangulate(&input).expect("general position input");

    // Element node numbers are only meaningful because vertex `i` of the output
    // is point `i` of the input. Nothing is reordered or deduplicated.
    assert_eq!(mesh.vertices(), input.as_slice());
}

#[test]
fn square_with_centre_gives_four_triangles() {
    let input = points(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.5, 0.5)]);
    let mesh = triangulate(&input).expect("general position input");

    assert_eq!(mesh.triangles().len(), 4);
    assert_valid_triangulation(&mesh, 1.0);

    // The hull is the square, so exactly the four sides are boundary edges.
    assert_eq!(mesh.boundary_edge_keys().len(), 4);
}

#[test]
fn triangle_count_follows_eulers_formula() {
    // Any triangulation of n points in general position, h of them on the convex
    // hull, has exactly 2n - h - 2 triangles. This holds whichever triangulation
    // is chosen, so it tests completeness without pinning down connectivity.
    let mut rng = Lcg::new(0xC0FFEE);
    let input = rng.points_in_unit_square(60);

    let mesh = triangulate(&input).expect("random points are in general position");

    let n = input.len();
    let h = convex_hull(&input).len();
    assert_eq!(mesh.triangles().len(), 2 * n - h - 2);
    assert_well_formed(&mesh);
}

#[test]
fn random_points_produce_a_valid_triangulation() {
    let mut rng = Lcg::new(0x5EED_2024);
    let input = rng.points_in_unit_square(40);

    let mesh = triangulate(&input).expect("random points are in general position");

    // The union of the triangles must be the convex hull, computed independently.
    assert_valid_triangulation(&mesh, convex_hull_area(&input));
}

#[test]
fn a_second_seed_also_produces_a_valid_triangulation() {
    let mut rng = Lcg::new(0x1234_5678_9ABC);
    let input = rng.points_in_unit_square(120);

    let mesh = triangulate(&input).expect("random points are in general position");
    assert_valid_triangulation(&mesh, convex_hull_area(&input));
}

#[test]
fn a_point_far_outside_the_others_is_still_included() {
    // Exercises the super triangle sizing: the bounding box is dominated by one
    // distant point, and the rest cluster near the origin.
    let input = points(&[
        (0.0, 0.0),
        (0.1, 0.0),
        (0.05, 0.1),
        (0.02, 0.03),
        (100.0, 100.0),
    ]);

    let mesh = triangulate(&input).expect("general position input");
    assert_valid_triangulation(&mesh, convex_hull_area(&input));
}
