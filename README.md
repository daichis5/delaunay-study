# delaunay-study

Rust の学習を兼ねて、ドロネー三角形分割 (Delaunay triangulation) を外部ライブラリに頼らず自作する教育用プロジェクトです。

計算幾何 (computational geometry) のアルゴリズムそのものより、**幾何述語 (geometric predicate) の符号規約**と**退化 (degeneracy)・浮動小数点誤差の扱い**を理解することに重点を置いています。

## 現在の実装範囲

| 項目 | 状態 |
| --- | --- |
| 2D 幾何述語 (`orient2d`, `incircle`) | 実装済み |
| 2D Bowyer–Watson 法 | 実装済み |
| CSV 入出力・CLI | 実装済み |
| Rust テスト・CI | 実装済み |
| 3D 幾何述語 (`orient3d`, `insphere`) | **未実装** |
| 3D 四面体分割 (tetrahedralization) | **未実装** |
| SciPy との比較 | **未実装** |
| Matplotlib / PyVista による可視化 | **未実装** |

3D と Python 側は未着手です。`--dimension 3` は「未実装」というエラーを返します。動いているように見せかけることはしません。

## 言語の役割分担

- **Rust**: アルゴリズム、幾何データ構造、幾何述語、メッシュ接続情報、CSV 入出力、CLI、単体テスト
- **Python**（今後）: SciPy による参照分割、Matplotlib による 2D 可視化、PyVista による 3D 可視化

PyO3 や FFI は導入しません。Rust と Python は CSV ファイルを介して連携します。

## セットアップ

Rust ツールチェインは `rust-toolchain.toml` で 1.92.0 に固定しています。`rustup` が入っていれば自動で解決されます。

```bash
make setup   # cargo fetch
make test    # cargo test
make lint    # cargo fmt --check と clippy -D warnings
```

## CLI の使い方

```bash
cargo run --bin delaunay -- \
  --dimension 2 \
  --input data/square_center_2d.csv \
  --output outputs/rust_2d.csv
```

`make run-2d` でも同じことができます。

### 入力 CSV

`id,x,y` のヘッダが必要です。列は名前で探すので順序は自由で、余分な列は無視されます。

```csv
id,x,y
0,0.0,0.0
1,1.0,0.0
2,1.0,1.0
3,0.0,1.0
4,0.5,0.5
```

**`id` は `0..n-1` の連続整数を昇順で並べる必要があります。** これを満たさない入力はエラーです。

この制約は意図的に厳しくしてあります。おかげで次の3つがすべて同じ番号になり、対応表 (mapping table) が一切不要になります。

- 入力 CSV の `id`
- 内部の頂点インデックス
- SciPy の `Delaunay.simplices` が参照する入力配列の位置

将来この制約を緩める場合は、そのときに対応表を導入します。

### 出力 CSV

```csv
element_id,node0,node1,node2
0,0,1,4
1,1,2,4
2,0,4,3
3,2,3,4
```

`node0..node2` は入力の `id` そのものです。三角形の頂点は必ず**反時計回り (counter-clockwise, CCW)** に並びます。

## 2D Bowyer–Watson 法の概要

逐次挿入法 (incremental insertion) の一種です。

1. 全入力点を内部に含む super triangle を作る
2. 点を1つずつ挿入する
3. 挿入点を外接円 (circumcircle) の内部に含む三角形を bad triangles として集める
4. bad triangles の辺のうち1回しか現れないものが、空洞の境界 (cavity boundary) になる
5. 各境界辺と挿入点から新しい三角形を作り、**その場で** CCW に再配向する
6. 全点の挿入後、super triangle の頂点を含む三角形を削除する

詳細と、手順5の「その場で」が重要な理由は [docs/algorithm.md](docs/algorithm.md) にあります。

## ドロネー条件 (Delaunay condition)

任意の三角形の外接円の内部に、他のどの点も入らないこと。これが定義であり、実装の正しさを測る基準です。

重要なのは「内部 (strictly inside)」という点です。**円周上 (cocircular) の点は違反ではありません。** 正方形の4頂点のように、複数の点がちょうど同一円周上にある配置では、ドロネー分割は**一意に定まりません**。どちらの対角線を選んでも正しい答えです。

したがって、他の実装と接続情報 (connectivity) が一致しないことは、必ずしも誤りを意味しません。

## 幾何述語と符号規約

```rust
orient2d(a, b, c) > 0   // a, b, c が反時計回り (CCW)
incircle(a, b, c, d) > 0 // d が a, b, c の外接円の内部（ただし a,b,c が CCW のときのみ）
```

`incircle` は行列式 (determinant) なので、引数の向きが反転すると符号も反転します。時計回り (clockwise, CW) の三角形に対して呼ぶと、内部と外部の判定が静かに入れ替わります。クラッシュせず、ただ間違ったメッシュができます。

このため本実装では、**メッシュ中の三角形は常に CCW である**という不変条件 (invariant) を保ち、向きの統一を「アルゴリズムの最後」ではなく「三角形を生成した瞬間」に行っています。

詳細は [docs/predicates.md](docs/predicates.md) を参照してください。

## 浮動小数点誤差と robust predicate

述語は通常の `f64` で行列式を計算しています。ほぼ退化した配置では減算で桁落ちが起き、ゼロ近傍の符号は信頼できません。

本実装は、これを許容誤差 (tolerance) で誤魔化していません。述語の中にも呼び出し側にも `if value.abs() < EPSILON` の類は書きません。理由は2つあります。

1. 絶対的な EPSILON は理論的に無意味です。`orient2d` は座標の2乗、`incircle` は4乗のオーダーでスケールするため、両者に共通の定数は存在しません。
2. しきい値でゼロと判定することは、退化を「なかったこと」にするだけで、解決していません。

代わりに、**符号をそのまま使い、ほぼ退化した入力の結果は保証しない**と明示します。許容誤差はテストの検証コードにのみ存在し、面積の比較のような本質的に近似的な判断にしか使いません。トポロジーを決めることはありません。

将来的には Shewchuk の adaptive precision 述語や [`robust`](https://crates.io/crates/robust) クレートへの置き換えでこの制限を解消できます。署名を合わせてあるので、差し替えは述語モジュール内に閉じます。

## 退化入力に対する契約 (contract)

### エラーになるもの

三角形分割が原理的に存在しない入力は、panic ではなく `Result` の `Err` を返します。

| 条件 | エラー |
| --- | --- |
| 3点未満 | `TooFewPoints` |
| 座標が NaN または無限大 | `NonFiniteCoordinate` |
| 座標が完全に一致する点が2つ以上 | `DuplicatePoint` |
| 全点が共線 (collinear) | `AllPointsCollinear` |

### 受理するが結果を保証しないもの

以下は処理を続行しますが、得られる接続情報の正しさを保証しません。

- 一部の点のみが共線
- 挿入点が既存の辺のちょうど上に載る
- 4点以上が共円 (cocircular)
- ほぼ共線・ほぼ共円

**「保証しない」の範囲は、思うより広いことに注意してください。** 「正しい答えが複数あるうちの1つを選ぶ」に留まらず、**凸包 (convex hull) を覆いきれない結果になることがあります。** 例えば正六角形の6頂点は厳密に同一円周上にあり、この入力では三角形が1つ欠けます。半径を `1e-15` だけずらして厳密な共円性を崩すと6個そろうので、これはアルゴリズムの誤りではなく `f64` 述語の限界です。

受理された入力について常に成り立つのは、局所的な構造だけです。インデックスが範囲内、要素内で頂点が重複しない、すべての要素が CCW。

この区別は `tests/degeneracy.rs` にそのままテストとして書かれています。「成功してもエラーでもよい」というテストは1つもありません。

## 性能

初版に性能要件はありません。各挿入で全三角形を走査して bad triangles を探す素朴な実装で、**おおむね二次時間 (O(n²))** です。可読性を優先した結果であり、意図的なものです。

将来の最適化余地:

- 空間分割による bad triangle 探索の高速化
- point location の改善（walk など）
- super triangle の頂点を数値ではなく記号的 (symbolic) に扱い、無限遠点として処理する

3つ目は性能ではなく正しさの改善で、後述の既知の制限を根本的に解消します。

## 既知の制限

- **super triangle の大きさが経験則です。** 入力点だけで作られる三角形の外接円の中に super triangle の頂点が入ると、その三角形が生成されず、凸包の縁に穴が残ります。凸包上の3点が共線に近づくほど外接円は無制限に大きくなるため、有限の倍率では保証になりません。現在は bounding box の 1e6 倍を採用しており、ランダム入力480ケース（10〜500点 × 80 seed）で失敗ゼロですが、これは余裕の問題であって証明ではありません。根本的な解決は上記の記号的処理です。
- 述語が robust ではありません（上記）。
- 3D と Python 側が未実装です。

## 今後の予定

1. SciPy による参照分割との比較と、Matplotlib による 2D 可視化
2. 3D 幾何述語と 3D Bowyer–Watson 法
3. PyVista による 3D 可視化
4. robust predicate への差し替えと、退化契約の見直し

## ライセンス

MIT ライセンス、または Apache License 2.0 のいずれかを選択して利用できます。

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)
