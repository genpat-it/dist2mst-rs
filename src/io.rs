use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Read sample IDs from a text file, one per line.
pub fn read_samples_of_interest(path: &Path) -> Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut samples = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            samples.push(trimmed);
        }
    }
    Ok(samples)
}

/// Write clusters to a TSV file, sorted by size (largest first).
pub fn write_clusters_tsv(
    clusters: &[Vec<String>],
    output_path: &Path,
    nwk_paths: Option<&HashMap<usize, String>>,
) -> Result<()> {
    let mut indexed: Vec<(usize, &Vec<String>)> = clusters.iter().enumerate().collect();
    indexed.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let mut file = fs::File::create(output_path)?;

    let mut header = String::from("Cluster\tSamples\tSize");
    if nwk_paths.is_some() {
        header.push_str("\tNWK_Path");
    }
    writeln!(file, "{header}")?;

    for (new_idx, (orig_idx, cluster)) in indexed.iter().enumerate() {
        let mut sorted = cluster.to_vec();
        sorted.sort();
        let line = format!("{}\t{}\t{}", new_idx + 1, sorted.join(","), cluster.len());
        if let Some(paths) = nwk_paths {
            if let Some(path) = paths.get(orig_idx) {
                writeln!(file, "{line}\t{path}")?;
            } else {
                writeln!(file, "{line}")?;
            }
        } else {
            writeln!(file, "{line}")?;
        }
    }

    Ok(())
}
