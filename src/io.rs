//! Reading point sets and writing element tables as CSV.
//!
//! # The `id` constraint
//!
//! An input file's `id` column must be exactly `0, 1, ..., n-1` in ascending
//! order. This is stricter than it needs to be, and deliberately so: it makes
//! the input `id`, the internal vertex index, and the position in the array
//! SciPy's `Delaunay.simplices` refers to all the same number. No lookup table
//! is needed anywhere, and the comparison scripts cannot silently mismatch rows.
//!
//! Supporting sparse or unordered ids means introducing that mapping, which is
//! left for a later version.

use std::path::Path;

use crate::delaunay2d::Mesh2;
use crate::error::{DelaunayError, Result};
use crate::geometry::Point2;

/// Reads a 2D point set from a CSV file with an `id,x,y` header.
///
/// Columns are located by name, so their order in the file does not matter and
/// additional columns are ignored. The `id` column must satisfy the constraint
/// described in the [module documentation](self).
pub fn read_points_2d(path: &Path) -> Result<Vec<Point2>> {
    let mut reader = csv::Reader::from_path(path)?;

    let headers = reader.headers()?.clone();
    let id_column = column_index(&headers, "id")?;
    let x_column = column_index(&headers, "x")?;
    let y_column = column_index(&headers, "y")?;

    let mut points = Vec::new();
    for (row, record) in reader.records().enumerate() {
        let record = record?;

        let id = field(&record, id_column, "id")?;
        if id.trim().parse::<usize>() != Ok(row) {
            return Err(DelaunayError::NonContiguousId {
                row,
                expected: row,
                found: id.to_string(),
            });
        }

        points.push(Point2::new(
            parse_coordinate(field(&record, x_column, "x")?, row, "x")?,
            parse_coordinate(field(&record, y_column, "y")?, row, "y")?,
        ));
    }

    Ok(points)
}

/// Writes the triangles of a mesh as an `element_id,node0,node1,node2` table.
///
/// The node numbers are vertex indices, which thanks to the `id` constraint are
/// also the `id` values of the input file.
pub fn write_triangles(path: &Path, mesh: &Mesh2) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(["element_id", "node0", "node1", "node2"])?;

    for (element_id, triangle) in mesh.triangles().iter().enumerate() {
        let [a, b, c] = triangle.nodes;
        writer.write_record([
            element_id.to_string(),
            a.to_string(),
            b.to_string(),
            c.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

/// Locates a column by header name.
fn column_index(headers: &csv::StringRecord, name: &'static str) -> Result<usize> {
    headers
        .iter()
        .position(|header| header.trim() == name)
        .ok_or(DelaunayError::MissingColumn { name })
}

/// Extracts a field that the header promised would be there.
fn field<'a>(record: &'a csv::StringRecord, index: usize, name: &'static str) -> Result<&'a str> {
    record
        .get(index)
        .ok_or(DelaunayError::MissingColumn { name })
}

/// Parses one coordinate, reporting where it came from if it is not a number.
fn parse_coordinate(value: &str, row: usize, column: &'static str) -> Result<f64> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| DelaunayError::InvalidNumber {
            row,
            column,
            value: value.to_string(),
        })
}
