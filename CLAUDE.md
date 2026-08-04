# delaunay-study

An educational implementation of 2D Delaunay triangulation and 3D Delaunay
tetrahedralization, written to learn Rust. The algorithms are implemented from
scratch in Rust. Python is used only for verification and visualization.

## Language policy

**English** for everything that lives with the code: source, identifiers, inline
comments, rustdoc doc comments, `CLAUDE.md`, config files, commit messages, and
pull request descriptions.

**Japanese** for `README.md` and everything under `docs/`. These double as study
notes for the author, so they are written in the author's first language.

Where the two meet, each side keeps its own language. The degenerate-input
contract, for example, is documented in English rustdoc on `DelaunayError` and
summarized in Japanese in the README.

### Terminology in the Japanese documents

Give the English term in parentheses at first use of any mathematical or
technical term. This keeps the prose searchable against the literature and
consistent with the identifiers in the code.

- ドロネー三角形分割 (Delaunay triangulation)
- 四面体分割 (tetrahedralization)
- 幾何述語 (geometric predicate)
- 外接円 (circumcircle) / 外接球 (circumsphere)
- 空円条件 (empty circumcircle condition)
- 符号付き面積 (signed area) / 符号付き体積 (signed volume)
- 反時計回り (counter-clockwise, CCW)
- 凸包 (convex hull)
- 退化 (degeneracy) / 一般位置 (general position)
- 共線 (collinear) / 共面 (coplanar) / 共円 (cocircular) / 共球 (cospherical)
- 逐次挿入法 (incremental insertion)

Repeat the parenthetical when a section is meant to be read standalone. Names
that appear verbatim in the code (`orient2d`, `insphere`, `DelaunayError`) are
used as-is, without a Japanese gloss.

## Division of labor

- **Rust**: algorithms, geometric data structures, geometric predicates, mesh
  connectivity, CSV I/O, CLI, unit tests
- **Python**: reference triangulation via SciPy, 2D plots via Matplotlib, 3D
  rendering via PyVista

Do not introduce PyO3 or any FFI. Rust and Python exchange data through CSV or
JSON files only.

## Invariants that must not be broken

### Sign conventions for geometric predicates

- `orient2d(a, b, c) > 0` means `a, b, c` are counter-clockwise (CCW)
- `orient3d(a, b, c, d) / 6` is the signed volume; positive means positively oriented
- `incircle` means "positive = `d` lies inside the circumcircle" **only when**
  `orient2d(a, b, c) > 0`
- `insphere` means "positive = `e` lies inside the circumsphere" **only when**
  `orient3d(a, b, c, d) > 0`

`incircle` and `insphere` depend on orientation. **Never call them on an element
whose orientation is not guaranteed.**

Consequently, orientation must be normalized **at the moment a new element is
created**, not as a final pass over the mesh. If orientation drifts during the
Bowyer-Watson insertion loop, bad-simplex detection silently inverts.

This matters most in 3D: the sorted vertex key used to count shared boundary
faces does not preserve face orientation, so every new tetrahedron must be
re-oriented with `orient3d` rather than inheriting the face's vertex order.

### Numerical error

- Use the **raw sign** of a predicate for topological decisions. Never test
  `|value| < EPSILON` to decide "zero", neither inside the predicate nor at the
  call site.
- Results for nearly-collinear, nearly-coplanar, nearly-cocircular, and
  nearly-cospherical inputs are **explicitly unguaranteed**.
- Tolerance constants exist **for verification only**. Never reuse one constant
  for both a topology-deciding predicate and an area/volume or
  empty-circumcircle check.

### Error handling

- Failures caused by input (too few points, NaN/infinity, duplicate coordinates,
  all points collinear, all points coplanar, malformed CSV) return `Err`
- Failures caused by our own bugs (cavity boundary not closed, newly created
  element with negative orientation) use `debug_assert!`
- Keep these two apart: never hide a bug behind `Result`, never panic on bad input
- `.unwrap()` and `.expect()` in library code are panics and fall under the same rule

### Input and output

- The `id` column of an input CSV must be the contiguous range `0..n-1`;
  anything else is an `Err`
- This guarantees that output `node0..` indices coincide with the input `id`,
  the internal array index, and SciPy's `simplices` values. Do not build a
  mapping table.

## Coding guidelines

- Avoid unnecessary abstraction; favor code a beginner can follow
- Do not lean on macros or advanced type-system features
- Never use `unsafe`
- Document functions and structs
- Make error types specific enough that callers can tell which condition failed
- Keep data structures separate from algorithms
- Do not force a shared abstraction over the 2D and 3D code paths
- Prefer readability over performance
- No performance requirements for the first version. Scanning every element to
  find bad simplices is fine. This is roughly quadratic, which is acceptable
  here. Do not implement spatial acceleration or improved point location; record
  them in the README as future optimization opportunities.

## Commands

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Python lives in `python/`, managed with `uv sync` / `uv run`.

## License

Dual-licensed under MIT OR Apache-2.0. Keep the `license` field in `Cargo.toml`
in sync.

## About the specification document

`objective.md` at the repository root is the original implementation brief. It is
**local-only and git-ignored**, and it is written in Japanese since it is not part
of the repository.

As implementation progresses, distill each section into the destination below,
then delete that section from `objective.md`. The file should end up empty and
be discarded.

| Content in `objective.md` | Destination |
| --- | --- |
| Sign conventions, numerical error policy | `docs/predicates.md` |
| 2D/3D procedures, super simplex, re-orientation timing | `docs/algorithm.md` |
| Role of the SciPy comparison, verification checks | `docs/validation.md` |
| Degenerate-input contract | rustdoc on `DelaunayError` and `triangulate()`, summarized in README |
| CLI, CSV format, `id` constraint | README and `--help` |
| List of property checks for tests | comments in the test code itself |
| Implementation guidelines | this file (already distilled) |
| Directory layout, priorities, completion criteria | discard; fold into PR descriptions |

Complete the distillation within the PR that implements the corresponding code.
Never defer it to "later".

## Git workflow

- GitHub uses **squash merge**. Only the PR title and description survive on
  `main`, so state the implemented scope, design decisions, and known limitations
  in the PR description.
- Split work into multiple local commits. `cargo test` must pass at every commit.
- One concern per PR. The initial scaffold is the only exception.
