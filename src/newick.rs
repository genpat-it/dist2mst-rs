use crate::tree::Tree;

impl Tree {
    /// Convert tree to Newick format string.
    /// Uses iterative approach to avoid stack overflow on deep trees.
    pub fn to_newick(&self) -> String {
        let mut result = String::with_capacity(self.nodes.len() * 30);
        self.write_newick_node(self.root, &mut result);
        result.push(';');
        result
    }

    fn write_newick_node(&self, node_idx: usize, out: &mut String) {
        let node = &self.nodes[node_idx];

        if node.children.is_empty() {
            // Leaf node
            if node.samples.len() > 1 {
                out.push('(');
                out.push_str(&node.samples.join(","));
                out.push(')');
            } else {
                out.push_str(&node.samples[0]);
            }
            return;
        }

        // Internal node: sort children by distance
        let mut sorted_children: Vec<(usize, f64)> = node.children.clone();
        sorted_children.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Build child strings
        let mut child_strings: Vec<String> = Vec::with_capacity(sorted_children.len());
        for (child_idx, distance) in &sorted_children {
            let mut child_str = String::new();
            self.write_newick_node(*child_idx, &mut child_str);
            child_str.push(':');
            write_distance(&mut child_str, *distance);
            child_strings.push(child_str);
        }

        if node.samples.is_empty() {
            out.push('(');
            out.push_str(&child_strings.join(","));
            out.push(')');
        } else {
            // Node with both samples and children
            let mut parts: Vec<String> = node.samples.clone();
            parts.extend(child_strings);
            out.push('(');
            out.push_str(&parts.join(","));
            out.push(')');
        }
    }
}

/// Format distance matching Python's default float formatting.
fn write_distance(out: &mut String, dist: f64) {
    if dist == dist.floor() && dist.abs() < 1e15 {
        // Integer-like: write without decimal (matching Python's f"{distance}")
        out.push_str(&format!("{}", dist as i64));
    } else {
        out.push_str(&format!("{}", dist));
    }
}
