//! The single error type used across this crate.
//!
//! The rule this crate follows is documented on [`DelaunayError`]: anything the
//! *caller* can be blamed for becomes an `Err`, while anything *we* can be
//! blamed for becomes a `debug_assert!`. Bugs are never hidden behind a
//! `Result`, and bad input never panics.

use std::fmt;

/// Every way a triangulation run can fail because of its input.
///
/// # What is and is not an error
///
/// These variants cover *input* problems only. A violated internal invariant
/// (an unclosed cavity boundary, a newly created element with negative
/// orientation) is a bug in this crate and is reported with `debug_assert!`
/// instead, because a caller cannot do anything useful about it.
///
/// # Degenerate input
///
/// Only *fully* degenerate configurations are rejected here. Inputs that are
/// merely awkward -- a few collinear points, a point that lands exactly on an
/// existing edge, four cocircular points, nearly collinear points -- are
/// accepted and processed, but the resulting connectivity is explicitly
/// **not guaranteed** to be unique or even correct. See `docs/predicates.md`.
#[derive(Debug)]
pub enum DelaunayError {
    /// Fewer points than the minimum needed to build a single element.
    TooFewPoints { given: usize, required: usize },

    /// A coordinate was NaN or infinite.
    NonFiniteCoordinate { index: usize, axis: &'static str },

    /// Two input points share exactly the same coordinates.
    DuplicatePoint { first: usize, second: usize },

    /// Every input point lies on a single line, so no triangle exists.
    AllPointsCollinear,

    /// The CSV header did not contain a required column.
    MissingColumn { name: &'static str },

    /// The `id` column was not the contiguous range `0..n-1` in ascending order.
    NonContiguousId {
        row: usize,
        expected: usize,
        found: String,
    },

    /// A field could not be parsed as a floating point number.
    InvalidNumber {
        row: usize,
        column: &'static str,
        value: String,
    },

    /// `--dimension` was given a value other than 2 or 3.
    UnsupportedDimension { given: u8 },

    /// A code path that is deliberately not implemented yet.
    NotImplemented { what: &'static str },

    /// An underlying I/O failure.
    Io(std::io::Error),

    /// An underlying CSV parsing or writing failure.
    Csv(csv::Error),
}

impl fmt::Display for DelaunayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewPoints { given, required } => {
                write!(f, "need at least {required} points, got {given}")
            }
            Self::NonFiniteCoordinate { index, axis } => {
                write!(
                    f,
                    "point {index} has a non-finite {axis} coordinate (NaN or infinity)"
                )
            }
            Self::DuplicatePoint { first, second } => {
                write!(f, "points {first} and {second} have identical coordinates")
            }
            Self::AllPointsCollinear => {
                write!(f, "all input points are collinear, so no triangle exists")
            }
            Self::MissingColumn { name } => {
                write!(f, "input CSV is missing the required column `{name}`")
            }
            Self::NonContiguousId {
                row,
                expected,
                found,
            } => write!(
                f,
                "row {row}: expected id `{expected}`, found `{found}`; \
                 the id column must be the contiguous range 0..n-1 in ascending order"
            ),
            Self::InvalidNumber { row, column, value } => {
                write!(f, "row {row}: column `{column}` is not a number: `{value}`")
            }
            Self::UnsupportedDimension { given } => {
                write!(f, "unsupported dimension {given}, expected 2 or 3")
            }
            Self::NotImplemented { what } => {
                write!(f, "{what} is not implemented yet")
            }
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Csv(e) => write!(f, "CSV error: {e}"),
        }
    }
}

impl std::error::Error for DelaunayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Csv(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DelaunayError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<csv::Error> for DelaunayError {
    fn from(e: csv::Error) -> Self {
        Self::Csv(e)
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, DelaunayError>;
