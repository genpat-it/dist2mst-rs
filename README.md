# dist2mst-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![GitHub release](https://img.shields.io/github/v/release/genpat-it/dist2mst-rs)](https://github.com/genpat-it/dist2mst-rs/releases)

Ultra-fast Minimum Spanning Tree construction from symmetric distance matrices. Rust rewrite of [dist2mst](https://github.com/genpat-it/dist2mst) with **~900x speedup**.

## Performance

| Matrix size | Python + Numba | Rust | Speedup |
|-------------|---------------|------|---------|
| 1,000 x 1,000 | 0.26s | **0.004s** | 65x |
| 5,000 x 5,000 | 2m 53s | **0.19s** | 912x |
| 20,000 x 20,000 | hours* | **3.4s** | - |

\* Estimated; Python version not practical at this scale.

**System:** 4x Intel Xeon Gold 6252N @ 2.30GHz (192 threads), Linux x86_64

### Why so fast?

1. **O(V²) Prim's with min_dist array** — the Python version runs O(V³) by scanning all (unused, used) pairs each step. The Rust version maintains a `min_dist[]` auxiliary array, reducing each step to O(V).
2. **Cache-friendly flat array** — the distance matrix is stored as a contiguous `Vec<f64>` in row-major order, enabling CPU prefetching and LLVM auto-vectorization.
3. **Memory-mapped I/O** — large TSV files are parsed directly from memory-mapped regions.
4. **Parallel TSV parsing** — rows are parsed into floats in parallel using rayon.
5. **Arena-based tree** — `Vec<TreeNode>` with index references instead of heap-allocated pointer trees.

## Features

- Numba-free, dependency-free binary (single static executable)
- Zero-distance grouping for identical sequences
- Newick format output
- Hierarchical clustering with distance threshold
- Targeted clustering from samples of interest
- Individual cluster NWK file export
- Drop-in replacement for Python dist2mst (same CLI interface)

## Installation

### From source

```bash
git clone https://github.com/genpat-it/dist2mst-rs.git
cd dist2mst-rs
RUSTFLAGS="-C target-cpu=native" cargo build --release
# Binary at target/release/dist2mst
```

### From releases

Download pre-built binaries from [Releases](https://github.com/genpat-it/dist2mst-rs/releases).

## Usage

```bash
# Basic MST construction
dist2mst input_matrix.tsv output_tree.nwk

# With clustering
dist2mst input_matrix.tsv output_tree.nwk \
  --cluster-threshold 10 \
  --cluster-output clusters.tsv

# Targeted clustering with NWK export
dist2mst input_matrix.tsv output_tree.nwk \
  --cluster-threshold 10 \
  --samples-of-interest my_samples.txt \
  --cluster-output clusters.tsv \
  --min-cluster-size 5 \
  --max-cluster-size 50 \
  --cluster-nwk-dir ./cluster_nwk_files

# Control threads and suppress output
dist2mst input_matrix.tsv output_tree.nwk --threads 16 --quiet
```

### Command-line Arguments

| Argument | Description |
|----------|-------------|
| `input_file` | TSV file containing the symmetric distance matrix |
| `output_file` | Output file for the Newick tree |
| `--cluster-threshold FLOAT` | Maximum path distance threshold for clusters |
| `--cluster-output FILE` | Output file for cluster data (TSV) |
| `--min-cluster-size INT` | Minimum samples to generate a cluster NWK file |
| `--max-cluster-size INT` | Maximum samples for generating a cluster NWK file |
| `--cluster-nwk-dir DIR` | Directory for individual cluster NWK files |
| `--samples-of-interest FILE` | Sample IDs for targeted clustering |
| `--threads N` | Number of threads (0=auto-detect) |
| `--quiet` | Suppress progress output |

### Input Format

Tab-separated distance matrix:

```
	Sample1	Sample2	Sample3
Sample1	0	5	8
Sample2	5	0	3
Sample3	8	3	0
```

## Algorithm

1. **Central node selection** — node with minimum sum of distances
2. **Prim's MST** — optimized O(V²) with min_dist auxiliary array
3. **Zero-distance grouping** — identical sequences merged into single tree nodes
4. **Newick serialization** — children sorted by distance for deterministic output
5. **BFS clustering** — clusters bounded by maximum path distance threshold

## Compatibility

dist2mst-rs is a drop-in replacement for [dist2mst](https://github.com/genpat-it/dist2mst) (Python). Same CLI flags, same input/output formats. The MST may differ in tie-breaking order (both produce valid minimum spanning trees with identical total weight).

## License

MIT License — see [LICENSE](LICENSE) for details.

## Contact

- Email: genpat@izs.it
- GitHub: https://github.com/genpat-it/dist2mst-rs
- Python version: https://github.com/genpat-it/dist2mst
