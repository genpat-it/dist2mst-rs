use crate::tree::Tree;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Graph representation of the MST for clustering.
/// Maps sample_name -> [(neighbor_name, edge_distance)]
type MstGraph = BTreeMap<String, Vec<(String, f64)>>;

/// Build a graph representation from the tree.
fn build_graph(tree: &Tree) -> MstGraph {
    let mut graph: MstGraph = BTreeMap::new();

    fn walk(tree: &Tree, node_idx: usize, parent_idx: Option<usize>, parent_dist: f64, graph: &mut MstGraph) {
        let node = &tree.nodes[node_idx];

        // Ensure all samples have entries
        for sample in &node.samples {
            graph.entry(sample.clone()).or_default();
        }

        // If parent exists, connect all samples in this node to all samples in parent
        if let Some(pidx) = parent_idx {
            let parent = &tree.nodes[pidx];
            for sample in node.samples.iter() {
                for parent_sample in parent.samples.iter() {
                    graph.entry(sample.clone()).or_default()
                        .push((parent_sample.clone(), parent_dist));
                    graph.entry(parent_sample.clone()).or_default()
                        .push((sample.clone(), parent_dist));
                }
            }
        }

        // Process children sorted for determinism
        let mut sorted_children: Vec<(usize, f64)> = node.children.clone();
        sorted_children.sort_by(|a, b| {
            let a_samples: Vec<&str> = tree.nodes[a.0].samples.iter().map(|s| s.as_str()).collect();
            let b_samples: Vec<&str> = tree.nodes[b.0].samples.iter().map(|s| s.as_str()).collect();
            a_samples.cmp(&b_samples)
        });

        for (child_idx, dist) in sorted_children {
            walk(tree, child_idx, Some(node_idx), dist, graph);
        }
    }

    walk(tree, tree.root, None, 0.0, &mut graph);
    graph
}

/// Find all clusters in the MST where max path distance <= threshold.
pub fn find_clusters(tree: &Tree, threshold: f64, quiet: bool) -> Vec<Vec<String>> {
    if !quiet {
        eprintln!("[+] Finding clusters with maximum path distance threshold of {threshold}...");
    }
    let start = std::time::Instant::now();
    let graph = build_graph(tree);

    let mut visited: HashSet<String> = HashSet::new();
    let mut clusters: Vec<Vec<String>> = Vec::new();

    let samples: Vec<String> = graph.keys().cloned().collect();

    for start_sample in &samples {
        if visited.contains(start_sample) {
            continue;
        }

        let mut cluster: Vec<String> = Vec::new();
        let mut cluster_visited: HashMap<String, f64> = HashMap::new();
        cluster_visited.insert(start_sample.clone(), 0.0);

        // BFS with priority (distance, name) for determinism
        let mut queue: BTreeSet<(OrderedF64, String)> = BTreeSet::new();
        queue.insert((OrderedF64(0.0), start_sample.clone()));

        while let Some((_, sample)) = queue.pop_first() {
            if visited.contains(&sample) {
                continue;
            }

            cluster.push(sample.clone());
            visited.insert(sample.clone());

            if let Some(neighbors) = graph.get(&sample) {
                for (neighbor, _edge_dist) in neighbors {
                    if visited.contains(neighbor) {
                        continue;
                    }

                    // Find shortest path distance from start to neighbor
                    let mut min_dist = f64::INFINITY;
                    if let Some(nb_neighbors) = graph.get(neighbor) {
                        for (s, dist) in nb_neighbors {
                            if let Some(&s_dist) = cluster_visited.get(s) {
                                let new_dist = s_dist + dist;
                                if new_dist < min_dist {
                                    min_dist = new_dist;
                                }
                            }
                        }
                    }

                    if min_dist <= threshold {
                        queue.insert((OrderedF64(min_dist), neighbor.clone()));
                        cluster_visited.insert(neighbor.clone(), min_dist);
                    }
                }
            }
        }

        if !cluster.is_empty() {
            cluster.sort();
            clusters.push(cluster);
        }
    }

    let elapsed = start.elapsed();
    if !quiet {
        eprintln!("[+] Found {} clusters in {:.2}s", clusters.len(), elapsed.as_secs_f64());
    }

    clusters
}

/// Find clusters starting from specified samples of interest.
pub fn find_clusters_from_samples(
    tree: &Tree,
    samples_of_interest: &[String],
    threshold: f64,
    quiet: bool,
) -> Vec<Vec<String>> {
    if !quiet {
        eprintln!(
            "[+] Finding clusters from {} samples of interest with threshold {threshold}...",
            samples_of_interest.len()
        );
    }
    let start = std::time::Instant::now();
    let graph = build_graph(tree);

    let mut valid_samples: Vec<&String> = Vec::new();
    for sample in samples_of_interest.iter() {
        if graph.contains_key(sample) {
            valid_samples.push(sample);
        } else if !quiet {
            eprintln!("[!] Warning: Sample {sample} not found in the MST, skipping...");
        }
    }
    valid_samples.sort();

    if valid_samples.is_empty() {
        if !quiet {
            eprintln!("[!] Error: None of the specified samples of interest were found.");
        }
        return Vec::new();
    }

    let mut clusters: Vec<Vec<String>> = Vec::new();

    for start_sample in &valid_samples {
        let mut cluster: Vec<String> = Vec::new();
        let mut distances: HashMap<String, f64> = HashMap::new();
        distances.insert((*start_sample).clone(), 0.0);

        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: BTreeSet<(OrderedF64, String)> = BTreeSet::new();
        queue.insert((OrderedF64(0.0), (*start_sample).clone()));

        while let Some((OrderedF64(dist), sample)) = queue.pop_first() {
            if visited.contains(&sample) {
                continue;
            }

            cluster.push(sample.clone());
            visited.insert(sample.clone());

            if let Some(neighbors) = graph.get(&sample) {
                for (neighbor, edge_dist) in neighbors {
                    if visited.contains(neighbor) {
                        continue;
                    }
                    let new_dist = dist + edge_dist;
                    if new_dist <= threshold {
                        queue.insert((OrderedF64(new_dist), neighbor.clone()));
                        distances.insert(neighbor.clone(), new_dist);
                    }
                }
            }
        }

        if !cluster.is_empty() {
            cluster.sort();
            clusters.push(cluster);
        }
    }

    let elapsed = start.elapsed();
    if !quiet {
        eprintln!(
            "[+] Found {} clusters from samples of interest in {:.2}s",
            clusters.len(),
            elapsed.as_secs_f64()
        );
    }

    clusters
}

/// Extract a subtree containing only the specified samples.
pub fn extract_subtree(tree: &Tree, cluster_samples: &HashSet<String>) -> Tree {
    let graph = build_graph(tree);

    let relevant: Vec<String> = graph
        .keys()
        .filter(|s| cluster_samples.contains(*s))
        .cloned()
        .collect();

    if relevant.is_empty() {
        let mut t = Tree::new();
        t.add_node(Vec::new());
        return t;
    }

    let mut new_tree = Tree::new();
    let root_idx = new_tree.add_node(vec![relevant[0].clone()]);
    new_tree.root = root_idx;

    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(relevant[0].clone());

    // Sample name -> node index in new_tree
    let mut sample_to_node: HashMap<String, usize> = HashMap::new();
    sample_to_node.insert(relevant[0].clone(), root_idx);

    while visited.len() < relevant.len() {
        // Find closest unvisited relevant sample to any visited sample
        let mut min_dist = f64::INFINITY;
        let mut closest_visited = None;
        let mut closest_new = None;

        for v_sample in visited.iter() {
            if let Some(neighbors) = graph.get(v_sample) {
                for (neighbor, dist) in neighbors {
                    if cluster_samples.contains(neighbor)
                        && !visited.contains(neighbor)
                        && *dist < min_dist
                    {
                        min_dist = *dist;
                        closest_visited = Some(v_sample.clone());
                        closest_new = Some(neighbor.clone());
                    }
                }
            }
        }

        if let (Some(v), Some(new)) = (closest_visited, closest_new) {
            let parent_node = sample_to_node[&v];
            let child_idx = new_tree.add_node(vec![new.clone()]);
            new_tree.nodes[parent_node].children.push((child_idx, min_dist));
            sample_to_node.insert(new.clone(), child_idx);
            visited.insert(new);
        } else {
            break;
        }
    }

    new_tree
}

/// Calculate maximum edge distance in a subtree.
pub fn max_distance(tree: &Tree) -> f64 {
    fn walk(tree: &Tree, node_idx: usize) -> f64 {
        let mut max = 0.0f64;
        for (child_idx, dist) in &tree.nodes[node_idx].children {
            max = max.max(*dist).max(walk(tree, *child_idx));
        }
        max
    }
    walk(tree, tree.root)
}

/// Wrapper for f64 that implements Ord for use in BTreeSet.
#[derive(Clone, Debug)]
struct OrderedF64(f64);

impl PartialEq for OrderedF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for OrderedF64 {}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}
