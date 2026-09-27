//! Tests for CSV reading and writing, including the `id` constraint.

mod common;

use std::path::{Path, PathBuf};

use common::assert_valid_triangulation;
use delaunay_study::DelaunayError;
use delaunay_study::delaunay2d::triangulate;
use delaunay_study::io::{read_points_2d, write_triangles};

/// A path inside the crate, so tests do not depend on the working directory.
fn data_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(name)
}

/// A scratch path under `target/`, which `make clean` and `cargo clean` own.
fn scratch_path(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-scratch");
    std::fs::create_dir_all(&path).expect("scratch directory is writable");
    path.join(name)
}

/// Writes `contents` to a scratch CSV file and returns its path.
fn write_temp_csv(name: &str, contents: &str) -> PathBuf {
    let path = scratch_path(name);
    std::fs::write(&path, contents).expect("scratch file is writable");
    path
}

#[test]
fn reads_the_bundled_sample_files() {
    let points = read_points_2d(&data_path("simple_2d.csv")).expect("bundled data parses");
    assert_eq!(points.len(), 3);
    assert_eq!(points[0].x, 0.0);
    assert_eq!(points[2].y, 1.0);
}

#[test]
fn the_random_sample_triangulates_cleanly() {
    let points = read_points_2d(&data_path("random_2d.csv")).expect("bundled data parses");
    let mesh = triangulate(&points).expect("the sample is in general position");
    assert_valid_triangulation(&mesh, common::convex_hull_area(&points));
}

#[test]
fn round_trips_through_the_output_format() {
    let points = read_points_2d(&data_path("square_center_2d.csv")).expect("bundled data parses");
    let mesh = triangulate(&points).expect("general position input");

    let output = scratch_path("round_trip_2d.csv");
    write_triangles(&output, &mesh).expect("output is writable");

    let written = std::fs::read_to_string(&output).expect("output is readable");
    let mut lines = written.lines();
    assert_eq!(lines.next(), Some("element_id,node0,node1,node2"));
    assert_eq!(lines.count(), mesh.triangles().len());
}

#[test]
fn creates_missing_output_directories() {
    let points = read_points_2d(&data_path("simple_2d.csv")).expect("bundled data parses");
    let mesh = triangulate(&points).expect("general position input");

    let nested = scratch_path("nested").join("deeper").join("out.csv");
    let _ = std::fs::remove_dir_all(nested.parent().unwrap().parent().unwrap());

    write_triangles(&nested, &mesh).expect("missing directories are created");
    assert!(nested.exists());
}

#[test]
fn a_missing_column_is_rejected() {
    let path = write_temp_csv("missing_column.csv", "id,x\n0,0.0\n");
    assert!(matches!(
        read_points_2d(&path),
        Err(DelaunayError::MissingColumn { name: "y" })
    ));
}

#[test]
fn columns_may_appear_in_any_order() {
    let path = write_temp_csv("reordered.csv", "y,id,x\n0.5,0,1.5\n");
    let points = read_points_2d(&path).expect("columns are located by name");
    assert_eq!(points[0].x, 1.5);
    assert_eq!(points[0].y, 0.5);
}

#[test]
fn ids_that_do_not_start_at_zero_are_rejected() {
    let path = write_temp_csv("one_based.csv", "id,x,y\n1,0.0,0.0\n2,1.0,0.0\n");
    assert!(matches!(
        read_points_2d(&path),
        Err(DelaunayError::NonContiguousId {
            row: 0,
            expected: 0,
            ..
        })
    ));
}

#[test]
fn ids_with_a_gap_are_rejected() {
    let path = write_temp_csv("gap.csv", "id,x,y\n0,0.0,0.0\n1,1.0,0.0\n3,0.0,1.0\n");
    assert!(matches!(
        read_points_2d(&path),
        Err(DelaunayError::NonContiguousId {
            row: 2,
            expected: 2,
            ..
        })
    ));
}

#[test]
fn out_of_order_ids_are_rejected() {
    let path = write_temp_csv("unordered.csv", "id,x,y\n0,0.0,0.0\n2,1.0,0.0\n1,0.0,1.0\n");
    assert!(matches!(
        read_points_2d(&path),
        Err(DelaunayError::NonContiguousId { row: 1, .. })
    ));
}

#[test]
fn an_unparsable_coordinate_is_rejected() {
    let path = write_temp_csv("not_a_number.csv", "id,x,y\n0,0.0,zero\n");
    assert!(matches!(
        read_points_2d(&path),
        Err(DelaunayError::InvalidNumber {
            row: 0,
            column: "y",
            ..
        })
    ));
}

/// A NaN in the file parses successfully, then fails validation with a position.
///
/// The two layers stay separate on purpose: the reader's job is the file format,
/// the algorithm's job is whether the geometry is usable.
#[test]
fn a_nan_coordinate_parses_but_fails_validation() {
    let path = write_temp_csv("nan.csv", "id,x,y\n0,0.0,0.0\n1,1.0,0.0\n2,NaN,1.0\n");
    let points = read_points_2d(&path).expect("NaN is a valid float literal");
    assert!(matches!(
        triangulate(&points),
        Err(DelaunayError::NonFiniteCoordinate {
            index: 2,
            axis: "x"
        })
    ));
}
