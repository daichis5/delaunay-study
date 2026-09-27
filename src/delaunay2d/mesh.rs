//! The triangle mesh produced by the 2D algorithm.

use std::collections::{HashMap, HashSet};

use crate::geometry::{Point2, Triangle, orient2d};

/// A set of vertices together with the triangles connecting them.
///
/// # Orientation invariant
///
/// Every triangle in a `Mesh2` built through [`Mesh2::push_oriented`] is
/// counter-clockwise. This is not cosmetic: `incircle` gives the wrong answer on
/// a clockwise triangle, so the insertion loop would silently misclassify
/// elements without it. See `docs/predicates.md`.
#[derive(Debug, Clone, Default)]
pub struct Mesh2 {
    vertices: Vec<Point2>,
    triangles: Vec<Triangle>,
}

impl Mesh2 {
    /// Creates a mesh with the given vertices and no triangles.
    pub fn new(vertices: Vec<Point2>) -> Self {
        Self {
            vertices,
            triangles: Vec::new(),
        }
    }

    /// The vertex coordinates, indexed by vertex number.
    pub fn vertices(&self) -> &[Point2] {
        &self.vertices
    }

    /// The triangles, each holding three indices into [`Mesh2::vertices`].
    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    /// Adds a triangle, reordering its vertices so that it is counter-clockwise.
    ///
    /// This is the *only* place triangles enter a mesh, which is what keeps the
    /// orientation invariant true at every point in time rather than only at the
    /// end of the algorithm.
    ///
    /// If the three points are exactly collinear the orientation is zero and
    /// there is nothing to fix; the degenerate triangle is stored as given. That
    /// can only happen for degenerate input, whose output this crate documents
    /// as unguaranteed.
    pub fn push_oriented(&mut self, a: usize, b: usize, c: usize) {
        let orientation = orient2d(self.vertices[a], self.vertices[b], self.vertices[c]);
        self.triangles.push(if orientation < 0.0 {
            Triangle::new(a, c, b)
        } else {
            Triangle::new(a, b, c)
        });
    }

    /// Keeps only the triangles for which `predicate` returns `true`.
    pub fn retain_triangles(&mut self, predicate: impl FnMut(&Triangle) -> bool) {
        self.triangles.retain(predicate);
    }

    /// Drops the trailing `count` vertices.
    ///
    /// Used to remove the super triangle's vertices once the algorithm is done.
    /// They are appended last precisely so that removing them needs no
    /// renumbering of the input vertices.
    pub fn truncate_vertices(&mut self, count: usize) {
        self.vertices.truncate(count);
    }

    /// The three corner points of a triangle, in its stored (counter-clockwise) order.
    pub fn triangle_points(&self, triangle: &Triangle) -> [Point2; 3] {
        let [a, b, c] = triangle.nodes;
        [self.vertices[a], self.vertices[b], self.vertices[c]]
    }

    /// The signed area of a triangle. Positive for a counter-clockwise triangle.
    pub fn signed_area(&self, triangle: &Triangle) -> f64 {
        let [a, b, c] = self.triangle_points(triangle);
        orient2d(a, b, c) / 2.0
    }

    /// The sum of the signed areas of all triangles.
    ///
    /// For a valid triangulation this equals the area of the convex hull, which
    /// is the cheapest global sanity check available. It is not sufficient on
    /// its own -- an overlap and a hole of equal size cancel out -- so pair it
    /// with [`Mesh2::edge_use_counts`].
    pub fn total_area(&self) -> f64 {
        self.triangles.iter().map(|t| self.signed_area(t)).sum()
    }

    /// How many triangles use each edge, keyed by the edge's normalized form.
    ///
    /// In a valid triangulation every count is 1 (a hull edge) or 2 (an interior
    /// edge). Any other value means the mesh overlaps itself or has a hole.
    pub fn edge_use_counts(&self) -> HashMap<(usize, usize), usize> {
        let mut counts = HashMap::new();
        for triangle in &self.triangles {
            for edge in triangle.edges() {
                *counts.entry(edge.key()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// The edges used by exactly one triangle, i.e. the boundary of the mesh.
    pub fn boundary_edge_keys(&self) -> Vec<(usize, usize)> {
        let mut keys: Vec<_> = self
            .edge_use_counts()
            .into_iter()
            .filter(|&(_, count)| count == 1)
            .map(|(key, _)| key)
            .collect();
        keys.sort_unstable();
        keys
    }

    /// The set of vertex indices referenced by at least one triangle.
    pub fn used_vertices(&self) -> HashSet<usize> {
        self.triangles
            .iter()
            .flat_map(|t| t.nodes)
            .collect::<HashSet<_>>()
    }
}
