//! Command line front end.
//!
//! ```text
//! delaunay --dimension 2 --input data/simple_2d.csv --output outputs/rust_2d.csv
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use delaunay_study::delaunay2d::triangulate;
use delaunay_study::error::{DelaunayError, Result};
use delaunay_study::io::{read_points_2d, write_triangles};

/// Triangulate a point set read from CSV.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Spatial dimension of the input. Only 2 is implemented.
    #[arg(short, long)]
    dimension: u8,

    /// Input CSV file. Requires an `id,x,y` header with ids `0..n-1`.
    #[arg(short, long)]
    input: PathBuf,

    /// Output CSV file for the resulting elements.
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();

    match run(&args) {
        Ok(count) => {
            println!("wrote {count} triangles to {}", args.output.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            // Print the whole chain so that a CSV parse failure names the file
            // and position, not just "CSV error".
            eprintln!("error: {error}");
            let mut source = std::error::Error::source(&error);
            while let Some(cause) = source {
                eprintln!("  caused by: {cause}");
                source = cause.source();
            }
            ExitCode::FAILURE
        }
    }
}

/// Runs one triangulation, returning how many elements were written.
fn run(args: &Args) -> Result<usize> {
    match args.dimension {
        2 => {
            let points = read_points_2d(&args.input)?;
            let mesh = triangulate(&points)?;
            write_triangles(&args.output, &mesh)?;
            Ok(mesh.triangles().len())
        }
        3 => Err(DelaunayError::NotImplemented {
            what: "3D tetrahedralization",
        }),
        given => Err(DelaunayError::UnsupportedDimension { given }),
    }
}
