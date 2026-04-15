use crate::error::{Dist2MstError, Result};
use memmap2::Mmap;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

pub struct DistMatrix {
    pub n: usize,
    pub names: Vec<String>,
    pub name_to_idx: HashMap<String, usize>,
    pub data: Vec<f64>, // row-major flat array, n*n
}

impl DistMatrix {
    #[inline(always)]
    pub fn get(&self, i: usize, j: usize) -> f64 {
        unsafe { *self.data.get_unchecked(i * self.n + j) }
    }

    #[inline(always)]
    pub fn row(&self, i: usize) -> &[f64] {
        &self.data[i * self.n..(i + 1) * self.n]
    }

    pub fn from_tsv(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let content = std::str::from_utf8(&mmap)
            .map_err(|e| Dist2MstError::Validation(format!("Invalid UTF-8: {e}")))?;

        let mut lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Err(Dist2MstError::EmptyMatrix);
        }

        // Parse header line — first cell may be empty
        let header = lines.remove(0);
        let names: Vec<String> = header.split('\t').skip(1).map(|s| s.trim().to_string()).collect();
        let n = names.len();

        if n == 0 {
            return Err(Dist2MstError::EmptyMatrix);
        }
        if lines.len() != n {
            return Err(Dist2MstError::NotSquare {
                rows: lines.len(),
                cols: n,
            });
        }

        // Parse rows in parallel
        let mut data = vec![0.0f64; n * n];
        let chunk_size = n; // each row is n floats

        let row_results: Vec<std::result::Result<Vec<f64>, Dist2MstError>> = lines
            .par_iter()
            .enumerate()
            .map(|(row_idx, line)| {
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() != n + 1 {
                    return Err(Dist2MstError::NotSquare {
                        rows: n,
                        cols: fields.len() - 1,
                    });
                }
                let mut row = Vec::with_capacity(n);
                for (col_idx, field) in fields.iter().skip(1).enumerate() {
                    let val: f64 = field.trim().parse().map_err(|_| Dist2MstError::Parse {
                        row: row_idx + 1,
                        col: col_idx,
                        msg: format!("cannot parse '{field}' as float"),
                    })?;
                    row.push(val);
                }
                Ok(row)
            })
            .collect();

        for (i, result) in row_results.into_iter().enumerate() {
            let row = result?;
            data[i * chunk_size..(i + 1) * chunk_size].copy_from_slice(&row);
        }

        let name_to_idx: HashMap<String, usize> =
            names.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();

        Ok(DistMatrix {
            n,
            names,
            name_to_idx,
            data,
        })
    }
}
