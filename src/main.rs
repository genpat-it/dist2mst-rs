use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use dist2mst::cluster;
use dist2mst::error::Dist2MstError;
use dist2mst::io;
use dist2mst::matrix::DistMatrix;
use dist2mst::mst;
use dist2mst::tree::Tree;

const VERSION: &str = "0.1.0";

#[derive(Parser)]
#[command(
    name = "dist2mst",
    version = VERSION,
    about = "Ultra-fast Minimum Spanning Tree construction from distance matrices"
)]
struct Cli {
    /// TSV file containing the distance matrix
    input_file: PathBuf,

    /// Output file for the Newick tree
    output_file: PathBuf,

    /// Maximum path distance threshold for clusters
    #[arg(long)]
    cluster_threshold: Option<f64>,

    /// Output file for cluster data (TSV)
    #[arg(long)]
    cluster_output: Option<PathBuf>,

    /// Minimum number of samples required to generate a cluster NWK file
    #[arg(long)]
    min_cluster_size: Option<usize>,

    /// Maximum number of samples for generating a cluster NWK file
    #[arg(long)]
    max_cluster_size: Option<usize>,

    /// Directory to store individual NWK files for clusters
    #[arg(long)]
    cluster_nwk_dir: Option<PathBuf>,

    /// File containing sample IDs to use as starting points for clusters
    #[arg(long)]
    samples_of_interest: Option<PathBuf>,

    /// Number of threads (0=auto)
    #[arg(long, default_value_t = 0)]
    threads: usize,

    /// Minimize progress output
    #[arg(long)]
    quiet: bool,
}

fn main() {
    let cli = Cli::parse();

    if !cli.quiet {
        eprintln!("dist2mst v{VERSION} (Rust)");
        eprintln!("Author: GenPat");
        eprintln!("Contact: genpat@izs.it");
        eprintln!("{}", "-".repeat(50));
    }

    // Configure thread pool
    if cli.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.threads)
            .build_global()
            .ok();
        if !cli.quiet {
            eprintln!("[+] Running with {} threads", cli.threads);
        }
    } else if !cli.quiet {
        eprintln!("[+] Running with {} threads (auto)", rayon::current_num_threads());
    }

    // Validate arguments
    if let Err(e) = validate_args(&cli) {
        eprintln!("[-] ERROR: {e}");
        std::process::exit(1);
    }

    if let Err(e) = run(cli) {
        eprintln!("[-] ERROR: {e}");
        std::process::exit(1);
    }
}

fn validate_args(cli: &Cli) -> Result<(), String> {
    if cli.min_cluster_size.is_some() && cli.cluster_nwk_dir.is_none() {
        return Err("--cluster-nwk-dir is required when --min-cluster-size is provided".into());
    }
    if cli.max_cluster_size.is_some() && cli.min_cluster_size.is_none() {
        return Err("--min-cluster-size is required when --max-cluster-size is provided".into());
    }
    if cli.cluster_nwk_dir.is_some() && cli.min_cluster_size.is_none() {
        return Err("--min-cluster-size is required when --cluster-nwk-dir is provided".into());
    }
    if (cli.min_cluster_size.is_some() || cli.cluster_nwk_dir.is_some())
        && cli.cluster_threshold.is_none()
    {
        return Err("--cluster-threshold is required for cluster NWK generation".into());
    }
    if let (Some(min), Some(max)) = (cli.min_cluster_size, cli.max_cluster_size) {
        if min > max {
            return Err("--min-cluster-size must be <= --max-cluster-size".into());
        }
    }
    Ok(())
}

fn run(cli: Cli) -> Result<(), Dist2MstError> {
    let total_start = std::time::Instant::now();

    // Read distance matrix
    if !cli.quiet {
        eprintln!("[+] Reading distance matrix from {:?}", cli.input_file);
    }
    let read_start = std::time::Instant::now();
    let matrix = DistMatrix::from_tsv(&cli.input_file)?;
    if !cli.quiet {
        eprintln!(
            "[+] Loaded {}x{} distance matrix in {:.2}s",
            matrix.n,
            matrix.n,
            read_start.elapsed().as_secs_f64()
        );
    }

    // Build MST
    let (center, edges) = mst::build_mst(&matrix, cli.quiet);

    // Build tree
    let tree = Tree::from_mst(center, &edges, &matrix.names);

    // Clustering
    if let Some(threshold) = cli.cluster_threshold {
        let clusters = if let Some(ref soi_path) = cli.samples_of_interest {
            if !cli.quiet {
                eprintln!("[+] Reading samples of interest from {soi_path:?}");
            }
            let soi = io::read_samples_of_interest(soi_path)?;
            if !cli.quiet {
                eprintln!("[+] Found {} samples of interest", soi.len());
            }
            cluster::find_clusters_from_samples(&tree, &soi, threshold, cli.quiet)
        } else {
            cluster::find_clusters(&tree, threshold, cli.quiet)
        };

        // Cluster statistics
        if !cli.quiet {
            let sizes: Vec<usize> = clusters.iter().map(|c| c.len()).collect();
            eprintln!("[+] Cluster statistics:");
            eprintln!("    Total clusters: {}", clusters.len());
            if !sizes.is_empty() {
                let avg: f64 = sizes.iter().sum::<usize>() as f64 / sizes.len() as f64;
                eprintln!("    Average cluster size: {avg:.2}");
                eprintln!("    Largest cluster: {} samples", sizes.iter().max().unwrap());
                eprintln!("    Smallest cluster: {} samples", sizes.iter().min().unwrap());
            }
        }

        // Generate NWK files for eligible clusters
        let mut cluster_nwk_paths: HashMap<usize, String> = HashMap::new();
        if let (Some(min_size), Some(ref nwk_dir)) = (cli.min_cluster_size, &cli.cluster_nwk_dir) {
            let max_size = cli.max_cluster_size.unwrap_or(usize::MAX);
            fs::create_dir_all(nwk_dir)?;

            let mut nwk_count = 0;
            for (idx, c) in clusters.iter().enumerate() {
                let size = c.len();
                if size >= min_size && size <= max_size {
                    let sample_set: HashSet<String> = c.iter().cloned().collect();
                    let subtree = cluster::extract_subtree(&tree, &sample_set);
                    let max_dist = cluster::max_distance(&subtree);
                    let unique_id = &uuid::Uuid::new_v4().to_string()[..8];
                    let filename = format!("cluster_{size}_{max_dist:.1}_{unique_id}.nwk");
                    let path = nwk_dir.join(&filename);

                    let newick = subtree.to_newick();
                    fs::write(&path, &newick)?;
                    cluster_nwk_paths.insert(idx, path.to_string_lossy().into_owned());
                    nwk_count += 1;
                }
            }
            if !cli.quiet {
                eprintln!("[+] Generated {nwk_count} NWK files for eligible clusters");
            }
        }

        // Write cluster TSV
        if let Some(ref cluster_output) = cli.cluster_output {
            let paths = if cli.min_cluster_size.is_some() {
                Some(&cluster_nwk_paths)
            } else {
                None
            };
            io::write_clusters_tsv(&clusters, cluster_output, paths)?;
            if !cli.quiet {
                eprintln!("[+] Cluster data written to {cluster_output:?}");
            }
        }
    }

    // Convert to Newick and write
    if !cli.quiet {
        eprintln!("[+] Converting tree to Newick format...");
    }
    let newick_start = std::time::Instant::now();
    let newick = tree.to_newick();
    if !cli.quiet {
        eprintln!(
            "[+] Converted to Newick format in {:.4}s",
            newick_start.elapsed().as_secs_f64()
        );
    }

    if !cli.quiet {
        eprintln!("[+] Writing Newick tree to {:?}", cli.output_file);
    }
    fs::write(&cli.output_file, &newick)?;

    let total = total_start.elapsed();
    if !cli.quiet {
        eprintln!("[+] MST processing completed in {:.2}s", total.as_secs_f64());
        eprintln!("[+] Final Newick tree saved to {:?}", cli.output_file);
        eprintln!("[+] dist2mst v{VERSION} (Rust) completed successfully");
    }

    Ok(())
}
