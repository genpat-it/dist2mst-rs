use crate::mst::MstEdge;

pub struct TreeNode {
    pub samples: Vec<String>,
    pub children: Vec<(usize, f64)>, // (child_node_index, edge_weight)
}

pub struct Tree {
    pub nodes: Vec<TreeNode>,
    pub root: usize,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            nodes: Vec::new(),
            root: 0,
        }
    }

    pub fn add_node(&mut self, samples: Vec<String>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(TreeNode {
            samples,
            children: Vec::new(),
        });
        idx
    }

    /// Build tree from MST edges with zero-distance grouping.
    /// Replicates the Python behavior exactly.
    pub fn from_mst(center: usize, edges: &[MstEdge], names: &[String]) -> Self {
        let n = names.len();
        let mut tree = Tree::new();
        let mut idx_to_node: Vec<Option<usize>> = vec![None; n];

        // Create root node
        let root = tree.add_node(vec![names[center].clone()]);
        tree.root = root;
        idx_to_node[center] = Some(root);

        for edge in edges {
            let parent_node_idx = idx_to_node[edge.from].unwrap();

            if edge.weight == 0.0 && tree.nodes[parent_node_idx].samples.len() == 1 {
                // Merge into parent (zero-distance grouping)
                tree.nodes[parent_node_idx]
                    .samples
                    .push(names[edge.to].clone());
                idx_to_node[edge.to] = Some(parent_node_idx);
            } else {
                // Create new child node
                let child = tree.add_node(vec![names[edge.to].clone()]);
                tree.nodes[parent_node_idx]
                    .children
                    .push((child, edge.weight));
                idx_to_node[edge.to] = Some(child);
            }
        }

        tree
    }

    /// Count total samples in the tree.
    pub fn sample_count(&self) -> usize {
        self.nodes.iter().map(|n| n.samples.len()).sum()
    }
}
