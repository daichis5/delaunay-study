//! The Bowyer-Watson incremental insertion algorithm in 2D.

use std::collections::{HashMap, HashSet};

use crate::delaunay2d::mesh::Mesh2;
use crate::error::{DelaunayError, Result};
use crate::geometry::{Point2, incircle, orient2d};

/// How far the super triangle extends, as a multiple of the input's bounding box.
///
/// # Why this is large, and why it is still only a heuristic
///
/// Containing every input point is the easy part; a factor of two would do. The
/// binding requirement is stronger: **no super triangle vertex may fall inside
/// the circumcircle of any triangle of input points.** When one does, that
/// triangle never forms, and deleting the super-touching triangles at the end
/// leaves a hole along the convex hull. The mesh still passes every local check
/// -- orientations, edge use counts, the empty circumcircle condition -- and
/// only its total area gives the loss away.
///
/// Circumcircles near the hull grow without bound as three hull points approach
/// collinearity, so no finite factor is a guarantee. It is a matter of margin.
/// Measured over 480 random cases (10 to 500 points, 80 seeds each), counting
/// runs whose total area missed the convex hull area:
///
/// | factor  | failures |
/// |---------|----------|
/// | 20      | many     |
/// | 1e3     | 9 / 480  |
/// | 1e4     | 1 / 480  |
/// | 1e5     | 0 / 480  |
/// | 1e6     | 0 / 480  |
///
/// Enlarging the triangle costs less than it appears to. Predicates on triangles
/// made only of input points never see these coordinates, so their precision is
/// untouched; only tests involving a super vertex use the large magnitudes, and
/// those are the ones the enlargement is meant to settle. In the limit the super
/// vertices behave like points at infinity, which is the construction this
/// approximates.
///
/// The principled fix is to treat them as exactly that -- handling the three
/// vertices symbolically instead of numerically -- which removes the constant
/// altogether. See `docs/algorithm.md`.
const SUPER_TRIANGLE_SCALE: f64 = 1.0e6;

/// The minimum number of points needed to form a triangle.
const MIN_POINTS: usize = 3;

/// Builds the Delaunay triangulation of `points`.
///
/// Returns a mesh whose vertices are `points` unchanged -- vertex `i` of the
/// result is `points[i]` -- and whose triangles are all counter-clockwise.
///
/// # Errors
///
/// The input is rejected outright when a triangulation cannot exist at all:
///
/// * [`DelaunayError::TooFewPoints`] -- fewer than three points
/// * [`DelaunayError::NonFiniteCoordinate`] -- a NaN or infinite coordinate
/// * [`DelaunayError::DuplicatePoint`] -- two points at identical coordinates
/// * [`DelaunayError::AllPointsCollinear`] -- every point on one line
///
/// # Unguaranteed input
///
/// Other awkward configurations are **accepted and processed**, but the
/// connectivity they produce is not guaranteed to be unique or even correct:
///
/// * some (not all) points collinear
/// * an inserted point landing exactly on an existing edge
/// * four or more cocircular points
/// * nearly collinear or nearly cocircular points
///
/// The first two are genuinely ambiguous -- more than one valid triangulation
/// exists and this crate picks one without a documented tie-break rule. The
/// last two are a floating point limitation: the predicates in
/// [`crate::geometry::predicates`] compute determinants in plain `f64`, whose
/// sign is unreliable near zero. Adaptive-precision predicates would fix the
/// numerical half of this; the ambiguous half needs a symbolic tie-break.
///
/// Be aware how far "unguaranteed" reaches. It is **not** limited to picking one
/// valid answer over another: the result may fail to be a triangulation of the
/// convex hull at all. Six points on a common circle, for example, come out with
/// a triangle missing, because `incircle` returns rounding noise where it should
/// return zero and the resulting cavity is not the star-shaped region this
/// algorithm assumes. What still holds in every accepted case is the local
/// structure -- indices in range, no repeated vertex, every element
/// counter-clockwise.
///
/// # Complexity
///
/// Roughly quadratic. Each insertion scans every existing triangle to find the
/// ones whose circumcircle contains the new point. This is the naive form of
/// the algorithm, kept for readability.
///
/// # Examples
///
/// ```
/// use delaunay_study::delaunay2d::triangulate;
/// use delaunay_study::geometry::Point2;
///
/// let points = vec![
///     Point2::new(0.0, 0.0),
///     Point2::new(1.0, 0.0),
///     Point2::new(0.0, 1.0),
/// ];
/// let mesh = triangulate(&points)?;
/// assert_eq!(mesh.triangles().len(), 1);
/// # Ok::<(), delaunay_study::DelaunayError>(())
/// ```
pub fn triangulate(points: &[Point2]) -> Result<Mesh2> {
    validate_input(points)?;

    let input_count = points.len();

    // The three super triangle vertices are appended *after* the input points,
    // so removing them at the end never renumbers anything the caller cares about.
    let mut vertices = points.to_vec();
    vertices.extend_from_slice(&super_triangle(points));

    let mut mesh = Mesh2::new(vertices);
    mesh.push_oriented(input_count, input_count + 1, input_count + 2);

    for index in 0..input_count {
        insert_point(&mut mesh, index);
    }

    // Anything still touching a super triangle vertex is an artifact of the
    // construction rather than part of the triangulation of the input.
    mesh.retain_triangles(|triangle| triangle.nodes.iter().all(|&node| node < input_count));
    mesh.truncate_vertices(input_count);

    Ok(mesh)
}

/// Inserts vertex `new_index`, which must already be present in the mesh's
/// vertex list, into the triangulation.
fn insert_point(mesh: &mut Mesh2, new_index: usize) {
    let new_point = mesh.vertices()[new_index];

    // Step 1: every triangle whose circumcircle contains the new point loses its
    // claim to be Delaunay. `incircle` is only meaningful here because every
    // triangle in the mesh is counter-clockwise.
    let bad: Vec<_> = mesh
        .triangles()
        .iter()
        .filter(|triangle| {
            let [a, b, c] = mesh.triangle_points(triangle);
            incircle(a, b, c, new_point) > 0.0
        })
        .copied()
        .collect();

    debug_assert!(
        !bad.is_empty(),
        "vertex {new_index} landed in no circumcircle; \
         it should at least be inside the triangle that contains it"
    );

    // Step 2: the cavity's boundary is the set of edges used by exactly one bad
    // triangle. Edges used twice are interior to the cavity and disappear with it.
    let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
    for triangle in &bad {
        for edge in triangle.edges() {
            *edge_counts.entry(edge.key()).or_insert(0) += 1;
        }
    }
    let boundary: Vec<_> = edge_counts
        .into_iter()
        .filter(|&(_, count)| count == 1)
        .map(|(key, _)| key)
        .collect();

    debug_assert!(
        !boundary.is_empty(),
        "cavity around vertex {new_index} has no boundary"
    );

    // Step 3: remove the bad triangles. Within one mesh a triangle is uniquely
    // identified by its vertex set, so the sorted form is a safe identity here.
    // Note that this sorted form is used *only* for identity -- never fed back
    // into a predicate, which would lose the orientation the algorithm depends on.
    let bad_keys: HashSet<[usize; 3]> = bad.iter().map(|t| t.sorted_nodes()).collect();
    mesh.retain_triangles(|triangle| !bad_keys.contains(&triangle.sorted_nodes()));

    // Step 4: fan the new point out to the cavity boundary. `push_oriented`
    // fixes the winding immediately, which is what keeps `incircle` valid on the
    // *next* insertion.
    for (a, b) in boundary {
        mesh.push_oriented(a, b, new_index);
    }
}

/// Rejects input that cannot produce a triangulation at all.
///
/// The checks run cheapest-first so that the reported error names the most
/// basic problem when several apply.
fn validate_input(points: &[Point2]) -> Result<()> {
    if points.len() < MIN_POINTS {
        return Err(DelaunayError::TooFewPoints {
            given: points.len(),
            required: MIN_POINTS,
        });
    }

    for (index, point) in points.iter().enumerate() {
        if !point.x.is_finite() {
            return Err(DelaunayError::NonFiniteCoordinate { index, axis: "x" });
        }
        if !point.y.is_finite() {
            return Err(DelaunayError::NonFiniteCoordinate { index, axis: "y" });
        }
    }

    let mut seen: HashMap<(u64, u64), usize> = HashMap::new();
    for (index, point) in points.iter().enumerate() {
        if let Some(&first) = seen.get(&point.exact_key()) {
            return Err(DelaunayError::DuplicatePoint {
                first,
                second: index,
            });
        }
        seen.insert(point.exact_key(), index);
    }

    // Duplicates are gone, so points[0] and points[1] span a genuine line. If no
    // other point leaves that line, every point is on it.
    let spans_a_triangle = points[2..]
        .iter()
        .any(|&c| orient2d(points[0], points[1], c) != 0.0);
    if !spans_a_triangle {
        return Err(DelaunayError::AllPointsCollinear);
    }

    Ok(())
}

/// Builds a triangle guaranteed to contain every input point strictly inside it.
///
/// The triangle is derived from the input's bounding box so that its size tracks
/// the data. See [`SUPER_TRIANGLE_SCALE`] for why it is not simply made huge.
fn super_triangle(points: &[Point2]) -> [Point2; 3] {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for point in points {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Validation has already rejected the only input that could give a
    // zero-sized box (every point identical), but a positive fallback keeps this
    // function total.
    let extent = (max_x - min_x).max(max_y - min_y);
    let extent = if extent > 0.0 { extent } else { 1.0 };

    let reach = SUPER_TRIANGLE_SCALE * extent;
    [
        Point2::new(center_x - reach, center_y - extent),
        Point2::new(center_x + reach, center_y - extent),
        Point2::new(center_x, center_y + reach),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn super_triangle_strictly_contains_the_bounding_box() {
        let points = [
            Point2::new(-3.0, 1.0),
            Point2::new(5.0, 1.0),
            Point2::new(0.0, 7.0),
        ];
        let [a, b, c] = super_triangle(&points);

        // Every corner of the input must be strictly left of all three edges of
        // the counter-clockwise super triangle.
        for point in points {
            assert!(orient2d(a, b, point) > 0.0);
            assert!(orient2d(b, c, point) > 0.0);
            assert!(orient2d(c, a, point) > 0.0);
        }
    }

    #[test]
    fn super_triangle_is_counter_clockwise() {
        let points = [
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.0, 1.0),
        ];
        let [a, b, c] = super_triangle(&points);
        assert!(orient2d(a, b, c) > 0.0);
    }
}
