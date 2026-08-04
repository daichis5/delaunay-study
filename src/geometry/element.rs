//! Mesh elements, and the distinction between oriented and normalized forms.
//!
//! Every element type here has two faces to it:
//!
//! * an **oriented** representation, whose vertex order carries geometric
//!   meaning (a [`Triangle`] is always counter-clockwise), and
//! * a **normalized** representation, whose vertex order is sorted so that it
//!   can be used as a map key or compared for identity.
//!
//! Mixing the two is the classic source of bugs in this algorithm, so the
//! normalized form is only ever produced through [`Edge::key`] and
//! [`Triangle::sorted_nodes`], never stored in place of the oriented one.

/// An undirected edge between two vertices, identified by index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub nodes: [usize; 2],
}

impl Edge {
    /// Creates an edge from two vertex indices, preserving the given order.
    pub fn new(a: usize, b: usize) -> Self {
        Self { nodes: [a, b] }
    }

    /// The normalized form of this edge: the smaller index first.
    ///
    /// Two edges traversed in opposite directions share the same key, which is
    /// what makes the cavity boundary extraction in
    /// [`crate::delaunay2d::bowyer_watson`] work.
    pub fn key(&self) -> (usize, usize) {
        let [a, b] = self.nodes;
        if a <= b { (a, b) } else { (b, a) }
    }
}

/// A triangle, identified by the indices of its three vertices.
///
/// Within a mesh produced by this crate, the vertex order is always
/// counter-clockwise, i.e. `orient2d` of the three vertices is positive.
/// See `docs/predicates.md` for why that matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle {
    pub nodes: [usize; 3],
}

impl Triangle {
    /// Creates a triangle from three vertex indices, preserving the given order.
    ///
    /// This does **not** enforce the counter-clockwise invariant. Use
    /// [`crate::delaunay2d::mesh::Mesh2::push_oriented`] when building a mesh.
    pub fn new(a: usize, b: usize, c: usize) -> Self {
        Self { nodes: [a, b, c] }
    }

    /// The three edges of this triangle, in traversal order.
    pub fn edges(&self) -> [Edge; 3] {
        let [a, b, c] = self.nodes;
        [Edge::new(a, b), Edge::new(b, c), Edge::new(c, a)]
    }

    /// The normalized form of this triangle: its vertex indices sorted.
    ///
    /// Only for comparison and lookup. It discards the orientation, so it must
    /// never be fed back into an orientation-dependent predicate.
    pub fn sorted_nodes(&self) -> [usize; 3] {
        let mut sorted = self.nodes;
        sorted.sort_unstable();
        sorted
    }

    /// Returns `true` if any of the three vertex indices is `index`.
    pub fn contains(&self, index: usize) -> bool {
        self.nodes.contains(&index)
    }
}
