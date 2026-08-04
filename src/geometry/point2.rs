//! A point in the plane.

/// A point in 2D space.
///
/// This is deliberately a plain data holder with public fields. Keeping it
/// transparent makes the determinant expressions in [`crate::geometry::predicates`]
/// read like the formulas they implement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    /// Creates a point from its coordinates.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Returns `true` when both coordinates are finite (neither NaN nor infinite).
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// A hashable key identifying the exact coordinates of this point.
    ///
    /// Used for duplicate detection. `f64` is not `Hash` or `Eq` because of NaN,
    /// so the raw bit patterns are used instead. Adding `0.0` first collapses
    /// `-0.0` onto `+0.0`, which otherwise have different bit patterns while
    /// comparing equal numerically.
    ///
    /// Callers must reject non-finite coordinates before using this, since two
    /// NaNs would produce equal keys while never comparing equal as numbers.
    pub fn exact_key(&self) -> (u64, u64) {
        ((self.x + 0.0).to_bits(), (self.y + 0.0).to_bits())
    }
}
