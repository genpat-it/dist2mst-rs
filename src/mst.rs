use crate::matrix::DistMatrix;

pub struct MstEdge {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
}

/// Find the central node (minimum sum of distances to all others).
pub fn find_central_node(matrix: &DistMatrix) -> usize {
    let n = matrix.n;
    let mut best_sum = f64::INFINITY;
    let mut best_idx = 0;

    for i in 0..n {
        let row = matrix.row(i);
        let sum: f64 = row.iter().sum();
        if sum < best_sum {
            best_sum = sum;
            best_idx = i;
        }
    }
    best_idx
}

/// Build MST using optimized Prim's algorithm with min_dist array.
/// O(V²) instead of the Python's O(V³).
pub fn build_mst(
    matrix: &DistMatrix,
    quiet: bool,
) -> (usize, Vec<MstEdge>) {
    let n = matrix.n;

    if !quiet {
        eprintln!("[+] Building MST for {} nodes", n);
    }

    let start = std::time::Instant::now();

    // Find central node
    let center = find_central_node(matrix);
    if !quiet {
        eprintln!(
            "[+] Selected node {} as central node (index {})",
            matrix.names[center], center
        );
    }

    let mut in_mst = vec![false; n];
    let mut min_dist = vec![f64::INFINITY; n];
    let mut min_from = vec![0usize; n];
    let mut edges = Vec::with_capacity(n.saturating_sub(1));

    // Initialize from center
    in_mst[center] = true;
    for j in 0..n {
        min_dist[j] = matrix.get(center, j);
        min_from[j] = center;
    }

    let report_interval = std::cmp::max(1, n / 10);

    for step in 1..n {
        // Find unused node with smallest min_dist
        let mut best = f64::INFINITY;
        let mut best_idx = 0;
        for j in 0..n {
            if !in_mst[j] && min_dist[j] < best {
                best = min_dist[j];
                best_idx = j;
            }
        }

        edges.push(MstEdge {
            from: min_from[best_idx],
            to: best_idx,
            weight: best,
        });
        in_mst[best_idx] = true;

        // Update min_dist for remaining nodes
        let new_row = matrix.row(best_idx);
        for j in 0..n {
            if !in_mst[j] && new_row[j] < min_dist[j] {
                min_dist[j] = new_row[j];
                min_from[j] = best_idx;
            }
        }

        if !quiet && step % report_interval == 0 {
            eprintln!(
                "[+] Added {}/{} nodes to MST ({:.1}%)",
                step + 1,
                n,
                ((step + 1) as f64 / n as f64) * 100.0
            );
        }
    }

    let elapsed = start.elapsed();
    if !quiet {
        eprintln!("[+] MST construction completed in {:.4}s", elapsed.as_secs_f64());
        eprintln!("[+] Added {} nodes to tree", n);
    }

    (center, edges)
}
